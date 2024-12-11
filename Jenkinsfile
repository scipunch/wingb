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
                sh 'docker ps | grep wingb | awk \'{print $1}\' | xargs -I {} docker stop {}'
                sh 'docker run --env-file $DOTENV_FILE --detach --publish 8010:8000 wingb:latest'
            }
        }
    }
}
