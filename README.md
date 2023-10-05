# multibooru-mirror

Architecture:

- there is a RabbitMQ server acting as the internal message queue
- acqisition processes periodically scrape boorus and create `Record` messages
- the `Record`s are sent to the RabbitMQ, or stored locally until it becomes available
- a single persistence process is responsible from drawing `Record`s from RabbitMQ and storing them in a central database

## Kube dependencies

There must be RabbitMQ cluster and topology operators and their CRDs installed.
Also, the following resources need to exist:

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: imageboards
---
kind: RabbitmqCluster
metadata:
    name: message-broker
    namespace: imageboards
---
apiVersion: rabbitmq.com/v1beta1
kind: Vhost
metadata:
  name: default-vhost
  namespace: imageboards
spec:
  name: "/"
  defaultQueueType: quorum
  rabbitmqClusterReference:
    name: message-broker
---
# This needs to be a SOCKS5 proxy server, allowing access to the imageboards.
apiVersion: v1
kind: Service
metadata:
  name: shadowsocks-client-service
  namespace: imageboards
spec:
  ports:
    - name: web
      port: 1080
      targetPort: socks

  selector:
    app: shadowsocks-client
```

When these resources are present, first deploy `common-resources.yaml`.
Each microservice's dependencies are provided in its own `deploy.yaml`.