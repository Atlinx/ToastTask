rm ~/.docker/config.json;
sudo docker-compose -f docker-compose.test.backend.yml down --volumes;
sudo docker-compose -f docker-compose.test.backend.yml up