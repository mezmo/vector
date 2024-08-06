library 'magic-butler-catalogue'

def WORKSPACE_PATH = "/tmp/workspace/${env.BUILD_TAG.replace('%2F', '/')}"
def DEFAULT_BRANCH = "master"
def PROJECT_NAME = "vector"
def CURRENT_BRANCH = [env.CHANGE_BRANCH, env.BRANCH_NAME]?.find{branch -> branch != null}

def slugify(str) {
  def s = str.toLowerCase()
  s = s.replaceAll(/[^a-z0-9\s-\/]/, "").replaceAll(/\s+/, " ").trim()
  s = s.replaceAll(/[\/\s]/, '-').replaceAll(/-{2,}/, '-')
  s
}

def BRANCH_BUILD = slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")

def CREDS = [
  string(
    credentialsId: 'github-api-token',
    variable: 'GITHUB_TOKEN'
  ),
  aws(credentialsId: 'aws',
    accessKeyVariable: 'AWS_ACCESS_KEY_ID',
    secretKeyVariable: 'AWS_SECRET_ACCESS_KEY'),
]

def NPMRC = [
    configFile(fileId: 'npmrc', variable: 'NPM_CONFIG_USERCONFIG')
]

pipeline {
  agent {
    node {
      label "rust-x86_64"
      customWorkspace(WORKSPACE_PATH)
    }
  }
  parameters {
    string(name: 'SANITY_BUILD', defaultValue: '', description: 'This a scheduled sanity build that skips releasing.')
  }
  triggers {
    parameterizedCron(
      // Cron hours are in GMT, so this is roughly 12-3am EST, depending on DST
      env.BRANCH_NAME == DEFAULT_BRANCH ? 'H H(5-6) * * * % SANITY_BUILD=true' : ''
    )
  }
  options {
    timestamps()
    disableConcurrentBuilds()
    timeout time: 90, unit: 'MINUTES'
    ansiColor 'xterm'
    withCredentials(CREDS)
  }
  environment {
    ENVIRONMENT_AUTOBUILD = 'false'
    ENVIRONMENT_TTY = 'false'
    CI = 'true'
    VECTOR_TARGET = "${BRANCH_BUILD}-target"
    VECTOR_CARGO_CACHE = "${BRANCH_BUILD}-cargo"
    VECTOR_RUSTUP_CACHE = "${BRANCH_BUILD}-rustup"
    TOP_COMMIT = sh(
      script: 'git log --pretty="format:%s by %cn" HEAD | head -1',
      returnStdout: true
    ).trim()
  }
  stages {
    stage('Release Tool') {
      steps {
        sh 'make release-tool'
      }
    }
    stage('Needs Testing?') {
      // Do not run the stages on a release commit, unless it's for a sanity build.
      when {
        anyOf {
          expression { !(env.TOP_COMMIT ==~ /^chore\(release\): \d+\.\d+\.\d+ \[skip ci\] by LogDNA Bot$/) }
          environment name: 'SANITY_BUILD', value: 'true'
        }
      }
      stages {
        stage('Lint and test release'){
          when {
            allOf {
                expression { !(env.TOP_COMMIT ==~ /^Merge remote-tracking branch.*$/) }
                expression { !(env.TOP_COMMIT ==~ /^Merge upstream.*$/) }
            }
          }
          tools {
            nodejs 'NodeJS 20'
          }
          environment {
            GIT_BRANCH = "${CURRENT_BRANCH}"
            // This is not populated on PR builds and is needed for the release dry runs
            BRANCH_NAME = "${CURRENT_BRANCH}"
            CHANGE_ID = ""
          }
          steps {
            script {
              configFileProvider(NPMRC) {
                sh 'npm ci'
                sh 'npm run release:dry'
              }
            }
            sh './release-tool lint'
            sh './release-tool test'
          }
        }
        stage('Unit test'){
          // Important: do one step serially since it'll be the one to prepare the testing container
          // and install the rust toolchain in it. Volume mounts are created here, too.
          steps {
            sh """
              make test ENVIRONMENT=true
            """
          }
        }
        stage('Checks') {
          // All `make ENVIRONMENT=true` steps should now use the existing container
          parallel {
            stage('check-clippy'){
              steps {
                sh """
                  make check-clippy ENVIRONMENT=true
                  make check-scripts ENVIRONMENT=true
                """
              }
            }
            stage('check-fmt'){
              steps {
                sh """
                  make check ENVIRONMENT=true
                  make check-fmt ENVIRONMENT=true
                """
              }
            }
            stage('check-deny'){
              steps {
                catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
                  sh """
                    make check-deny ENVIRONMENT=true
                  """
                }
              }
            }
            stage('image test') {
              when {
                changeRequest() // Only do this during PRs because it can take 30 minutes
              }
              steps {
                script {
                  def semver = npm.semver()
                  def pkg_version = "${semver.version}+${BRANCH_BUILD}"
                  buildx.build(
                    project: PROJECT_NAME
                  , push: false
                  , tags: [BRANCH_BUILD]
                  , dockerfile: "distribution/docker/mezmo/Dockerfile"
                  , args: [RELEASE_VERSION: pkg_version]
                  )
                }
              }
            }
          }
        }
        stage('Feature build and publish') {
          when {
            expression {
              CURRENT_BRANCH ==~ /feature\/LOG-\d+/
            }
          }
          steps {
            script {
              def tag = slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")
              def semver = npm.semver()
              def pkg_version = "${semver.version}+${tag}"
              buildx.build(
                project: PROJECT_NAME
              , push: true
              , tags: [tag]
              , dockerfile: "distribution/docker/mezmo/Dockerfile"
              , args: [RELEASE_VERSION: pkg_version]
              )
            }
            sh './release-tool clean'
            sh './release-tool build'
            sh './release-tool publish'
          }
        }
        stage('Release Commit') {
          when {
            branch DEFAULT_BRANCH
            not {
              environment name: 'SANITY_BUILD', value: 'true'
            }
          }
          tools {
            nodejs 'NodeJS 20'
          }
          steps {
            script {
              configFileProvider(NPMRC) {
                sh 'npm ci'
                sh 'npm run release'
              }
            }
          }
        }
      }
    }
    stage('Publish') {
      when {
        allOf {
          branch DEFAULT_BRANCH
          expression { env.TOP_COMMIT ==~ /^chore\(release\): \d+\.\d+\.\d+ \[skip ci\] by LogDNA Bot$/ }
          not {
            environment name: 'SANITY_BUILD', value: 'true'
          }
        }
      }
      steps {
        script {
          def tag = sh (
            script: "./release-tool debug-RELEASE_VERSION",
            returnStdout: true
          ).split(' = ')[1].trim()

          buildx.build(
            project: PROJECT_NAME
          , push: true
          , tags: [tag]
          , dockerfile: "distribution/docker/mezmo/Dockerfile"
          , args: [RELEASE_VERSION: tag]
          )
        }
        sh './release-tool clean'
        sh './release-tool build'
        sh './release-tool publish'
      }
    }
  }
  post {
    always {
      // Clear disk space by removing the test container and all of its volumes.
      // The container and volumes are unique to the current build, so there should be no "in use" errors.
      sh 'make environment-clean'

      script {
        if (env.SANITY_BUILD == 'true') {
          notifySlack(
            currentBuild.currentResult,
            [
              channel: '#pipeline-bots',
              tokenCredentialId: 'qa-slack-token'
            ],
            "`${PROJECT_NAME}` sanity build took ${currentBuild.durationString.replaceFirst(' and counting', '')}."
          )
        }
      }
    }
  }
}
