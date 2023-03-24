def WORKSPACE_PATH = "/tmp/workspace/${env.BUILD_TAG.replace('%2F', '/')}"

pipeline {
    agent {
        node {
            label "rust-x86_64"
            customWorkspace(WORKSPACE_PATH)
        }
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
