pipeline {
    agent any
    
    environment {
        DOTENV_FILE = credentials('podcastio-wingb-env')
    }

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
                sh 'make docker-run'
            }
        }
    }
}
