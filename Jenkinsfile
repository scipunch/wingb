pipeline {
    agent any

    stages {
        stage('📦 Build') {
            steps {
                echo 'Building..'
                docker build -t wingb .
            }
        }
        stage('🚀 Deploy') {
            steps {
                echo 'Deploying....'
            }
        }
    }
}