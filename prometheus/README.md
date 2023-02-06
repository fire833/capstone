
# Prometheus

## Setup

On the first run, you'll need to tell helm where to find the prometheus chart:
```
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
```

## Installation

```
helm install [RELEASE_NAME] -f values.yml prometheus-community/prometheus 
```

This will install prometheus and override some configuration with the values.yml in this directory,
disabling alertmanager and pushgateway and pointing prometheus at the exporter service


# Prometheus Adapter

## Setup

Add the helm community repository as described in the prometheus setup

## Installation


```
helm install [RELEASE_NAME] -f adapter.yml prometheus-community/prometheus-adapter 
```

This will install the prometheus adapter, overwriting some configuration with values in `adapter.yml`