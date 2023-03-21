package main

import "github.com/fire833/capstone/exporter/cmd/grid_exporter/app"

var (
	Version   string = "unknown"
	Commit    string = "unknown"
	BuildTime string = "unknown"
)

func main() {
	cmd := app.NewGridExporterCommand(Version, BuildTime, Commit)
	if e := cmd.Execute(); e != nil {
		cmd.Help()
	}
}
