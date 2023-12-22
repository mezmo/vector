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
    timeout time: 1, unit: 'HOURS'
    ansiColor 'xterm'
    withCredentials(CREDS)
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
    stage('Setup') {
      steps {
        sh 'make release-tool'
      }
    }
    stage('Check'){
      steps {
        sh """
          make check ENVIRONMENT=true
          make check-fmt ENVIRONMENT=true
        """
      }
    }
    stage('Lint and test release'){
      tools {
        nodejs 'NodeJS 16'
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
    stage('Feature build and publish') {
      when {
        expression {
          CURRENT_BRANCH ==~ /feature\/LOG-\d+/
        }
      }
      steps {
        script {
          def tag = slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")
          buildx.build(
            project: PROJECT_NAME
          , push: true
          , tags: [tag]
          , dockerfile: "distribution/docker/mezmo/Dockerfile"
          )
        }
        sh './release-tool clean'
        sh './release-tool build'
        sh './release-tool publish'
      }
    }
    stage('Release and publish') {
      when {
        branch DEFAULT_BRANCH
        not {
          environment name: 'SANITY_BUILD', value: 'true'
        }
      }
      tools {
        nodejs 'NodeJS 16'
      }
      steps {
        script {
          configFileProvider(NPMRC) {
            sh 'npm ci'
            sh 'npm run release'
          }

          def tag = sh (
            script: "./release-tool debug-RELEASE_VERSION",
            returnStdout: true
          ).split(' = ')[1].trim()

          buildx.build(
            project: PROJECT_NAME
          , push: true
          , tags: [tag]
          , dockerfile: "distribution/docker/mezmo/Dockerfile"
          )
        }
        sh './release-tool clean'
        sh './release-tool build'
        sh './release-tool publish'
      }
    }
  }
}
