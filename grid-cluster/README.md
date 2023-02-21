# selenium-grid-cluster

![Version: 0.1.2](https://img.shields.io/badge/Version-0.1.2-informational?style=flat-square) ![Type: application](https://img.shields.io/badge/Type-application-informational?style=flat-square) ![AppVersion: 4.8.0](https://img.shields.io/badge/AppVersion-4.8.0-informational?style=flat-square)

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
| nodes | object | `{"concurrency":1,"image":"docker.io/selenium/node-chrome:latest","maxReplicas":3,"minReplicas":1,"replicas":1,"resources":{"limits":{"cpu":"1","memory":"1Gi"}},"scalingUtilizationPercentage":80,"stabilizationWindowSeconds":30}` | Node management options for each selenium grid node managed by this chart. |
| nodes.concurrency | int | `1` | Specify the maximum concurrency level that you want for each node pod. |
| nodes.image | string | `"docker.io/selenium/node-chrome:latest"` | Specify the image for the node instances. |
| nodes.maxReplicas | int | `3` | Specify the maximum number of replicas for the selenium node pod within the cluster. |
| nodes.minReplicas | int | `1` | Specify the minimum number of replicas for the selenium node pod within the cluster. |
| nodes.replicas | int | `1` | Specify the number of selenium node pods you want spun up as a base case. |
| nodes.resources.limits.cpu | string | `"1"` | Specify the maximum cpu for every selenium node within the cluster. |
| nodes.resources.limits.memory | string | `"1Gi"` | Specify the maximum memory for every selenium node within the cluster. |
| nodes.scalingUtilizationPercentage | int | `80` | Utilization percentage target for scaling number of nodes up or down |
| nodes.stabilizationWindowSeconds | int | `30` | Specify the scaledown stabilization window for scaling node instances. |
| prometheus | object | `{"alertmanager":{"enabled":false},"kube-state-metrics":{"enabled":false},"prometheus-node-exporter":{"enabled":false},"prometheus-pushgateway":{"enabled":false},"server":{"persistentVolume":{"enabled":false}},"serverFiles":{"prometheus.yml":{"scrape_configs":[{"job_name":"grid_exporter","scrape_interval":"1s","static_configs":[{"targets":["exportersvc:8000"]}]}]}}}` | Override values for the prometheus dependency chart. |
| prometheus-adapter | object | `{"logLevel":5,"prometheus":{"path":"/","port":80,"url":"http://pv1-prometheus-server"},"rules":{"default":false,"external":[{"metricsQuery":"selenium_grid_num_nodes","resources":{"namespaced":false},"seriesQuery":"selenium_grid_num_nodes"},{"metricsQuery":"selenium_grid_num_sessions","name":{"as":"selenium_grid_num_sessions"},"resources":{"namespaced":false},"seriesQuery":"selenium_grid_num_sessions"},{"metricsQuery":"selenium_grid_max_sessions","name":{"as":"selenium_grid_max_sessions"},"resources":{"namespaced":false},"seriesQuery":"selenium_grid_max_sessions"},{"metricsQuery":"(avg_over_time(selenium_grid_num_sessions_aggregated[10s]) / selenium_grid_max_sessions_aggregated)* 100","name":{"as":"selenium_grid_session_util_percent"},"resources":{"namespaced":false},"seriesQuery":"{__name__=~\"^selenium_grid_.*_sessions_aggregated$\"}"}]}}` | Override values for the prometheus-adapter chart. |

----------------------------------------------
Autogenerated from chart metadata using [helm-docs v1.11.0](https://github.com/norwoodj/helm-docs/releases/v1.11.0)