library 'magic-butler-catalogue'

def PROJECT_NAME = 'vector'
def DEFAULT_BRANCH = 'master'
def CURRENT_BRANCH = [env.CHANGE_BRANCH, env.BRANCH_NAME]?.find { branch -> branch != null }

pipeline {
    agent {
        node {
            label "rust-x86_64"
            customWorkspace "${PROJECT_NAME}-${BUILD_NUMBER}"
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
            steps {
                sh """
                    make check ENVIRONMENT=true
                """
            }
        }
        stage('Unit test'){
            steps {
                sh """
                    make test ENVIRONMENT=true
                """
            }
        }
        stage('Build image and publish') {
            when {
                branch pattern: "v\\d+\\.\\d+.\\d+.\\d+", comparator: "REGEXP"
            }
            steps {
                sh 'make mezmo-build-image BUILD_VERSION=${BRANCH_NAME}'
                sh 'make mezmo-publish-image BUILD_VERSION=${BRANCH_NAME}'
            }
        }
    }
}