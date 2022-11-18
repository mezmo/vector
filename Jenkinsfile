pipeline {
    agent {
        node {
            label "rust-x86_64"
            customWorkspace("/tmp/workspace/${env.BUILD_TAG}")
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
