# selenium-grid-cluster

![Version: 0.4.0](https://img.shields.io/badge/Version-0.4.0-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 4.8.0](https://img.shields.io/badge/AppVersion-4.8.0-informational?style=flat-square)

Helm chart for deploying a dynamically-scalable Selenium Grid cluster and associated infrastructure.

## Maintainers

| Name | Email | Url |
| ---- | ------ | --- |
| Kendall Tauser | <kendall.tauser@ou.edu> | <https://github.com/fire833> |
| Aaron Pierce |  |  |

## Requirements

| Repository | Name | Version |
|------------|------|---------|
| https://prometheus-community.github.io/helm-charts | prometheus | 19.6.1 |
| https://prometheus-community.github.io/helm-charts | prometheus-adapter | 4.1.1 |

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| exporter | object | `{"image":"ghcr.io/fire833/capstone/grid-exporter:latest","resources":{"limits":{"cpu":"250m","memory":"30Mi"}}}` | Configuration for the selenium grid exporter |
| exporter.image | string | `"ghcr.io/fire833/capstone/grid-exporter:latest"` | Specify the image for the exporter instances. |
| exporter.resources.limits.cpu | string | `"250m"` | Specify the maximum cpu for the exporter instance within the cluster. |
| exporter.resources.limits.memory | string | `"30Mi"` | Specify the maximum memory for the exporter instance within the cluster. |
| hub | object | `{"image":"docker.io/selenium/hub:4.8.0","resources":{"limits":{"cpu":"1","memory":"1Gi"}}}` | Hub management options for the hub node managed by this chart.  |
| hub.image | string | `"docker.io/selenium/hub:4.8.0"` | Specify the image for the hub instance. |
| hub.resources.limits.cpu | string | `"1"` | Specify the maximum cpu for the hub instance within the cluster. |
| hub.resources.limits.memory | string | `"1Gi"` | Specify the maximum memory for the hub instance within the cluster. |
| nodes | object | `{"chrome":{"concurrency":1,"enabled":true,"image":"docker.io/selenium/node-chrome:latest","maxReplicas":3,"minReplicas":1,"replicas":1,"scalingUtilizationPercentage":80,"stabilizationWindowSeconds":30},"edge":{"concurrency":1,"enabled":true,"image":"docker.io/selenium/node-edge:latest","maxReplicas":3,"minReplicas":1,"replicas":1,"scalingUtilizationPercentage":80,"stabilizationWindowSeconds":30},"firefox":{"concurrency":1,"enabled":true,"image":"docker.io/selenium/node-firefox:latest","maxReplicas":3,"minReplicas":1,"replicas":1,"scalingUtilizationPercentage":80,"stabilizationWindowSeconds":30},"resources":{"limits":{"cpu":"1","memory":"1Gi"}}}` | Node management options for each selenium grid node managed by this chart. |
| nodes.chrome.concurrency | int | `1` | Specify the maximum concurrency level that you want for each node pod. |
| nodes.chrome.enabled | bool | `true` | Toggle whether chrome nodes should be spun up with the cluster. |
| nodes.chrome.image | string | `"docker.io/selenium/node-chrome:latest"` | Specify the image for the chrome node instances. |
| nodes.chrome.maxReplicas | int | `3` | Specify the maximum number of replicas for the selenium node pod within the cluster. |
| nodes.chrome.minReplicas | int | `1` | Specify the minimum number of replicas for the selenium node pod within the cluster. |
| nodes.chrome.replicas | int | `1` | Specify the number of chrome node pods you want spun up as a base case. |
| nodes.chrome.scalingUtilizationPercentage | int | `80` | Utilization percentage target for scaling number of chrome nodes up or down. |
| nodes.chrome.stabilizationWindowSeconds | int | `30` | Specify the scaledown stabilization window for scaling chrome node instances. |
| nodes.edge.concurrency | int | `1` | Specify the maximum concurrency level that you want for each node pod. |
| nodes.edge.enabled | bool | `true` | Toggle whether edge nodes should be spun up with the cluster. |
| nodes.edge.image | string | `"docker.io/selenium/node-edge:latest"` | Specify the image for the edge node instances. |
| nodes.edge.maxReplicas | int | `3` | Specify the maximum number of replicas for the edge node pod within the cluster. |
| nodes.edge.minReplicas | int | `1` | Specify the minimum number of replicas for the edge node pod within the cluster. |
| nodes.edge.replicas | int | `1` | Specify the number of edge node pods you want spun up as a base case. |
| nodes.edge.scalingUtilizationPercentage | int | `80` | Utilization percentage target for scaling number of edge nodes up or down. |
| nodes.edge.stabilizationWindowSeconds | int | `30` | Specify the scaledown stabilization window for scaling edge node instances. |
| nodes.firefox.concurrency | int | `1` | Specify the maximum concurrency level that you want for each node pod. |
| nodes.firefox.enabled | bool | `true` | Toggle whether firefox nodes should be spun up with the cluster. |
| nodes.firefox.image | string | `"docker.io/selenium/node-firefox:latest"` | Specify the image for the firefox node instances. |
| nodes.firefox.maxReplicas | int | `3` | Specify the maximum number of replicas for the firefox node pod within the cluster. |
| nodes.firefox.minReplicas | int | `1` | Specify the minimum number of replicas for the firefox node pod within the cluster. |
| nodes.firefox.replicas | int | `1` | Specify the number of firefox node pods you want spun up as a base case. |
| nodes.firefox.scalingUtilizationPercentage | int | `80` | Utilization percentage target for scaling number of firefox nodes up or down. |
| nodes.firefox.stabilizationWindowSeconds | int | `30` | Specify the scaledown stabilization window for scaling firefox node instances. |
| nodes.resources.limits.cpu | string | `"1"` | Specify the maximum cpu for every selenium node within the cluster. |
| nodes.resources.limits.memory | string | `"1Gi"` | Specify the maximum memory for every selenium node within the cluster. |
| prometheus | object | `{"alertmanager":{"enabled":false},"kube-state-metrics":{"enabled":false},"prometheus-node-exporter":{"enabled":false},"prometheus-pushgateway":{"enabled":false},"server":{"persistentVolume":{"enabled":false}},"serverFiles":{"prometheus.yml":{"scrape_configs":[{"job_name":"grid_exporter","scrape_interval":"1s","static_configs":[{"targets":["exportersvc:8000"]}]}]}}}` | Override values for the prometheus dependency chart. |
| prometheus-adapter | object | `{"logLevel":5,"prometheus":{"path":"/","port":80,"url":"http://{{ .Release.Name }}-prometheus-server"},"rules":{"default":false,"external":[{"metricsQuery":"selenium_grid_num_nodes","resources":{"namespaced":false},"seriesQuery":"selenium_grid_num_nodes"},{"metricsQuery":"selenium_grid_num_sessions","name":{"as":"selenium_grid_num_sessions"},"resources":{"namespaced":false},"seriesQuery":"selenium_grid_num_sessions"},{"metricsQuery":"selenium_grid_max_sessions","name":{"as":"selenium_grid_max_sessions"},"resources":{"namespaced":false},"seriesQuery":"selenium_grid_max_sessions"},{"metricsQuery":"(avg_over_time(selenium_grid_num_sessions_aggregated[10s]) / selenium_grid_max_sessions_aggregated)* 100","name":{"as":"selenium_grid_session_util_percent"},"resources":{"namespaced":false},"seriesQuery":"{__name__=~\"^selenium_grid_.*_sessions_aggregated$\"}"}]}}` | Override values for the prometheus-adapter chart. |

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.11.0](https://github.com/norwoodj/helm-docs/releases/v1.11.0)
