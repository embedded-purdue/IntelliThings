// One MQTT connection per virtual node, so each gets its own Last Will on
// intellithings/<node_id>/status — exactly how three real nodes look to Home Assistant.
#pragma once

const char *mqtt_broker_uri(void);  // "mqtt://<host>:<port>"
void mqtt_nodes_start(void);
