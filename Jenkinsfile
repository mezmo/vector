library 'magic-butler-catalogue'

def WORKSPACE_PATH = "/tmp/workspace/${env.BUILD_TAG.replace('%2F', '/')}"
def DEFAULT_BRANCH = "master"
def PROJECT_NAME = "vector"
def CURRENT_BRANCH = currentBranch()
def DOCKER_REPO = "docker.io/mezmo"
def TRIGGER_PATTERN = '.*@logdnabot.*'

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

def RELEASE_COMMIT_TITLE = /release: \d{4}-\d{2}-\d{2}, Version \d+\.\d+\.\d+ \[skip ci\] by LogDNA Bot$/

pipeline {
  agent {
    node {
      label "ec2-fleet"
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
    issueCommentTrigger(TRIGGER_PATTERN)
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
    RUST_BACKTRACE = 'full'
    RUST_LOG = 'debug'
    RUST_TEST_NOCAPTURE = '1'
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
    stage('Release Tool') {
      steps {
        sh 'make release-tool'
      }
    }
    stage('Needs Testing?') {
      // Do not run the stages on a release commit, unless it's for a sanity build.
      when {
        anyOf {
          expression { !(env.TOP_COMMIT ==~ RELEASE_COMMIT_TITLE) }
          environment name: 'SANITY_BUILD', value: 'true'
        }
      }
      stages {
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
              , docker_repo: DOCKER_REPO
              )
            }
            sh './release-tool clean'
            sh "./release-tool build APP_VERSION='" + slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}") + "'"
            sh "./release-tool publish RELEASE_VERSION='" + slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}") + "'"
            archiveArtifacts artifacts: 'output/'
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
