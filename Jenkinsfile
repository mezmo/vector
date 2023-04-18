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
              stage('Test build container image') {
                  when {
                    not {
                      branch pattern: "(^main|^master|v\\d+\\.\\d+.\\d+.\\d+(-[a-z_\\-0-9]+)?)", comparator: "REGEXP"
                    }
                  }
                  steps {
                    script {
                      buildx.build(
                        project: PROJECT_NAME
                      , push: false
                      , tags: [slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")]
                      , dockerfile: "distribution/docker/mezmo/Dockerfile"
                      )
                    }
                  }
              }
            }
        }
        stage('Build image and publish') {
            when {
              branch pattern: "(v\\d+\\.\\d+.\\d+.\\d+(-[a-z_\\-0-9]+)?)|(feature\\/LOG-\\d+)", comparator: "REGEXP"
            }

            steps {
              script {
                def tag = CURRENT_BRANCH
                if (CURRENT_BRANCH ==~ /feature\/LOG-\d+/) {
                  tag = slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")
                }
                buildx.build(
                  project: PROJECT_NAME
                , push: true
                , tags: [tag]
                , dockerfile: "distribution/docker/mezmo/Dockerfile"
                )
              }
            }
        }
    }
}
