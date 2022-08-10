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
    }
}