# multibooru-mirror

Architecture:

- there is a RabbitMQ server acting as the internal message queue
- acqisition processes periodically scrape boorus and create `Record` messages
- the `Record`s are sent to the RabbitMQ, or stored locally until it becomes available
- a single persistence process is responsible from drawing `Record`s from RabbitMQ and storing them in a central database