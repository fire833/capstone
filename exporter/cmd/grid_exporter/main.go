package main

import "github.com/fire833/capstone/exporter/cmd/grid_exporter/app"

func main() {
	cmd := app.NewGridExporterCommand()
	if e := cmd.Execute(); e != nil {
		cmd.Help()
	}
}
