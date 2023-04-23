
chart:
	helm package grid-cluster/ -d docs/
	helm repo index docs/
	tar cvf docs/aws_terraform.tar.gz terraform/aws/*
	tar cvf docs/gcp_terraform.tar.gz terraform/gcp/*
	tar cvf docs/azure_terraform.tar.gz terraform/azure/*
