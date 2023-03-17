
chart:
	helm package grid-cluster/ -d docs/
	helm repo index docs/
