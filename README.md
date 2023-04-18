
# CS-4273 Project 1 - Distributed, Scalable, Cloud-Agnostic Selenium Testing

The project provides a solution for running scalable Selenium grids on-premises and across multiple cloud providers, which consists of 3 parts:

1. A [Helm chart](https://helm.sh/docs/topics/charts/) to deploy an immediately operational and scalable [Selenium grid](https://www.selenium.dev/documentation/grid/applicability/) to a pre-existing [Kubernetes cluster](https://kubernetes.io/docs/concepts/overview/)

2. A set of [terraform definitions](https://developer.hashicorp.com/terraform/intro) to create a cluster on a managed kubernetes service (supporting [AWS](https://docs.aws.amazon.com/eks/latest/userguide/what-is-eks.html), [GCP](https://cloud.google.com/kubernetes-engine/docs/concepts/kubernetes-engine-overview#:~:text=GKE%20is%20a%20Google%2Dmanaged,in%2Dhouse%20cluster%20management%20system.), and [Azure](https://learn.microsoft.com/en-us/azure/aks/intro-kubernetes)), deploy the Helm chart to it, and automatically configure each provider's cluster autoscaling solution.

3. The Hub Router - a spec-compliant Selenium intermediate node which distributes tests among other Selenium grids and provides a central place for observability, allowing for ergonomic operation of multiple Selenium grids.

---
## Project Structure

`grid-cluster/` contains the Helm chart which deploys an immediately operational and scalable Selenium grid to a pre-existing Kubernetes cluster.
This chart is hosted on GitHub Pages at `https://fire833.github.io/capstone/index.yaml`.
To build and deploy a new version of the chart, the `Makefile` at the top level of the repository contains commands to do so.

`exporter/` contains a [Prometheus exporter](https://prometheus.io/docs/instrumenting/exporters/) to generate utilization metrics for the Selenium grid.
These metrics are used to automatically scale the Selenium nodes based on per-browser demand.
The exporter generates metrics like the number of actively running Firefox sessions, or the maximum number of Firefox sessions which could be running, which can be used to decide when the cluster should automatically spin up more Firefox nodes. 
This exporter is used exclusively by the Helm chart, and is accessed by pulling down a published image.
When the repository is tagged with a new version, a GitHub action defined in `.github/workflows/exporter-ci.yaml` will build the exporter's container image and publish it,
to be available for the Helm chart.

`terraform/` contains the Terraform definitions to deploy a Kubernetes cluster on your choice of cloud provider's managed Kubernetes service, deploys our helm chart to it, and configures that cloud provider's cluster autoscaling solution.
Deployment instructions for each individual cloud provider can be found in the README in the corresponding folder.

`hub_router_warp/` contains the source code for the Hub Router, which implements a spec-compliant Selenium intermediate node to distribute tests between multiple grids, and provides a unified web interface for configuring the router itself and viewing aggregated information about all registered grids.

`.github/` contains GitHub action definitions for automated deployment tasks, like running test suites and deploying container images and helm charts.

`docs/` does not contain documentation. It is the root of the GitHub pages site to make certain artifacts available, like helm charts.
This folder gets its name because GitHub pages will only serve the entire repository, or a top-level /docs folder.

`example-selenium-tests/` contains a convenient Selenium test to run while testing.