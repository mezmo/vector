def WORKSPACE_PATH = "/tmp/workspace/${env.BUILD_TAG.replace('%2F', '/')}"
def DEFAULT_BRANCH = "master"
def PROJECT_NAME = "vector"

pipeline {
    agent {
        node {
            label "rust-x86_64"
            customWorkspace(WORKSPACE_PATH)
        }
    }
    parameters {
      string(name: 'SANITY_BUILD', defaultValue: '', description: 'Is this a scheduled sanity build that skips releasing?')
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
        timeout time: 1, unit: 'HOURS'
        ansiColor 'xterm'
    }
    environment {
        ENVIRONMENT_AUTOBUILD = 'false'
        ENVIRONMENT_TTY = 'false'
    }
    post {
      always {
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
    stages {
        stage('Check'){
          parallel {
            stage('Check'){
                steps {
                    sh """
                        make check ENVIRONMENT=true
                    """
                }
            }
            stage('Style'){
                steps {
                    sh """
                        make check-fmt ENVIRONMENT=true
                        make check-style ENVIRONMENT=true
                    """
                }
            }
          }
        }
        stage('Lint and Test'){
            parallel {
              stage('Lint'){
                steps {
                  sh """
                    make check-clippy ENVIRONMENT=true
                    make check-scripts ENVIRONMENT=true
                  """
                }
              }
              stage('Deny'){
                steps {
                  catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
                    sh """
                      make check-deny ENVIRONMENT=true
                    """
                  }
                }
              }
              stage('Unit test'){
                steps {
                  sh """
                    make test ENVIRONMENT=true
                  """
                }
              }
            }
        }
        stage('Build image and publish') {
            when {
                branch pattern: "v\\d+\\.\\d+.\\d+.\\d+(-[a-z_\\-0-9]+)?", comparator: "REGEXP"
            }
            steps {
                sh 'make mezmo-build-image BUILD_VERSION=${BRANCH_NAME}'
                sh 'make mezmo-publish-image BUILD_VERSION=${BRANCH_NAME}'
            }
        }
    }
}
