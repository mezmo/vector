library 'magic-butler-catalogue'

def WORKSPACE_PATH = "/tmp/workspace/${env.BUILD_TAG.replace('%2F', '/')}"
def DEFAULT_BRANCH = "master"
def PROJECT_NAME = "vector"
def CURRENT_BRANCH = currentBranch()
def DOCKER_REPO = "docker.io/mezmohq"

def BRANCH_BUILD = slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")

def CREDS = [
  string(
    credentialsId: 'github-api-token',
    variable: 'GITHUB_TOKEN'
  ),
  aws(
    credentialsId: 'aws',
    accessKeyVariable: 'AWS_ACCESS_KEY_ID',
    secretKeyVariable: 'AWS_SECRET_ACCESS_KEY'
  ),
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
    stage('Validate PR Author') {
      when {
        expression { env.CHANGE_FORK }
        not {
          triggeredBy 'issueCommentCause'
        }
      }
      steps {
        error("A maintainer needs to approve this PR for CI by commenting")
      }
    }
    stage('Needs Testing?') {
      // Do not run the stages on a release commit, unless it's for a sanity build.
      when {
        anyOf {
          expression { !(env.TOP_COMMIT ==~ /^release: 20\d\d-\d\d-\d\d, Version \d+\.\d+\.\d+ \[skip ci\] by LogDNA Bot$/) }
          environment name: 'SANITY_BUILD', value: 'true'
        }
      }
      stages {
        stage('Lint and test release'){
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
                sh 'npm ci --ignore-scripts'
                sh 'npm run commitlint'
                if (!env.CHANGE_FORK) {
                  sh 'npm run release:dry'
                }
              }
            }
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
                  buildx.build(
                    project: PROJECT_NAME
                  , push: false
                  , tags: [BRANCH_BUILD]
                  , dockerfile: "distribution/docker/mezmo/Dockerfile"
                  , docker_repo: DOCKER_REPO
                  )
                }
              }
            }
          }
        } // End Checks

        stage('Feature build and publish') {
          when {
            expression {
              CURRENT_BRANCH ==~ /feature\/LOG-\d+/
            }
          }
          steps {
            script {
              def feature_tag = slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")
              buildx.build(
                project: PROJECT_NAME
              , push: true
              , tags: [feature_tag]
              , dockerfile: "distribution/docker/mezmo/Dockerfile"
              , docker_repo: DOCKER_REPO
              )
            }
          }
        } // End Feature build and publish

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
        } // End release commit

      } // End stages

    } // End Needs testing

    stage('Publish') {
      when {
        allOf {
          branch DEFAULT_BRANCH
          expression { env.TOP_COMMIT ==~ /^release: 20\d\d-\d\d-\d\d, Version \d+\.\d+\.\d+ \[skip ci\] by LogDNA Bot$/ }
          not {
            environment name: 'SANITY_BUILD', value: 'true'
          }
        }
      }
      tools {
        nodejs 'NodeJS 20'
      }
      steps {
        script {
          buildx.build(
            project: PROJECT_NAME
          , push: true
          , tags: [semver.version]
          , dockerfile: "distribution/docker/mezmo/Dockerfile"
          , docker_repo: DOCKER_REPO
          )
        }
      }
    } // End Publish
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
  } // End post
} // End pipeline
