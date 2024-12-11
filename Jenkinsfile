pipeline {
    agent any

    stages {
        stage('📦 Build') {
            steps {
                echo 'Building..'
                sh 'make docker-build'
            }
        }
        stage('🚀 Deploy') {
            steps {
                echo 'Deploying....'
            }
        }
    }
}