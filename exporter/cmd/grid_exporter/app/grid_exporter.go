package app

import (
	"net"
	"net/http"
	"strconv"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

func NewGridExporterCommand() *cobra.Command {
	var port uint16
	var hubhost string

	cmd := &cobra.Command{
		Use:     "grid-exporter",
		Short:   "A simple exporter to export metrics from Selenium grid clusters.",
		Version: "0.0.1",

		Long: "",

		RunE: func(cmd *cobra.Command, args []string) error {
			reg := prometheus.NewRegistry()

			reg.MustRegister(NewGridCollector(hubhost))

			handler := promhttp.HandlerFor(reg, promhttp.HandlerOpts{})

			listener, e := net.Listen("tcp", ":"+strconv.Itoa(int(port)))
			if e != nil {
				return e
			}

			return http.Serve(listener, handler)
		},
	}

	set := pflag.NewFlagSet("exporter", pflag.ExitOnError)

	set.Uint16Var(&port, "port", 9000, "Specify the port the exporter will listen on.")
	set.StringVar(&hubhost, "hub", "http://127.0.0.1:4444", "Specify the full hostname of the remote Grid Hub you want to export metrics from.")

	return cmd
}
