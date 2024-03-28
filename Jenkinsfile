library 'magic-butler-catalogue'

def WORKSPACE_PATH = "/tmp/workspace/${env.BUILD_TAG.replace('%2F', '/')}"
def DEFAULT_BRANCH = "master"
def PROJECT_NAME = "vector"
def CURRENT_BRANCH = currentBranch()
def DOCKER_REPO = "docker.io/mezmohq"

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
            sh 'npm run release:dry'
          }
        }
      }
    }

    stage('vdev Check'){
      when {
        changeRequest() // Only do this during PRs. It's about a 15-min wait.
      }
      steps {
        sh """
          make check ENVIRONMENT=true
          make check-fmt ENVIRONMENT=true
        """
      }
    }

    stage('Code'){
      parallel {
        stage('Lint'){
          steps {
            sh """
              make check-clippy ENVIRONMENT=true
              make check-scripts ENVIRONMENT=true
            """
          }
        }
        stage('Check Deny'){
          when {
            changeRequest() // PRs only to speed up dev flows. These can be fixed then if they're actionable.
          }
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
            changeRequest() // Only do this during PRs because it can take 30 minutes
          }
          steps {
            script {
              buildx.build(
                project: PROJECT_NAME
              , push: false
              , tags: [slugify("${CURRENT_BRANCH}-${BUILD_NUMBER}")]
              , dockerfile: "distribution/docker/mezmo/Dockerfile"
              , docker_repo: DOCKER_REPO
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
    }

    stage('Release and publish') {
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
          def version_before = npm.semver().version
          configFileProvider(NPMRC) {
            sh 'npm ci'
            sh 'npm run release'
          }

          def semver = npm.semver()
          if (version_before != semver.version) {
            buildx.build(
              project: PROJECT_NAME
            , push: true
            , tags: [semver.version]
            , dockerfile: "distribution/docker/mezmo/Dockerfile"
            , docker_repo: DOCKER_REPO
            )
          }
        }
      }
    }
  }
}
