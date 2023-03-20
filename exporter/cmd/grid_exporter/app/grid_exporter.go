package app

import (
	"fmt"
	"net"
	"net/http"
	"runtime"
	"strconv"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

func NewGridExporterCommand(version, date, commit string) *cobra.Command {
	var port uint16
	var hubhost string

	cmd := &cobra.Command{
		Use:   "grid-exporter",
		Short: "A simple exporter to export metrics from Selenium grid clusters.",

		Long: "",

		RunE: func(cmd *cobra.Command, args []string) error {

			fmt.Printf(`
            _     _                                  _            
  __ _ _ __(_) __| |       _____  ___ __   ___  _ __| |_ ___ _ __ 
 / _' | '__| |/ _  |_____ / _ \ \/ / '_ \ / _ \| '__| __/ _ \ '__|
| (_| | |  | | (_| |_____|  __/>  <| |_) | (_) | |  | ||  __/ |   
 \__, |_|  |_|\__,_|      \___/_/\_\ .__/ \___/|_|   \__\___|_|   
 |___/                             |_|                            

Version: %s
Built on: %s
Commit: %s
Go version: %s
OS: %s
Arch: %s

`,
				version, date, commit, runtime.Version(), runtime.GOOS, runtime.GOARCH)

			reg := prometheus.NewRegistry()

			reg.MustRegister(NewGridCollector(hubhost))

			handler := promhttp.HandlerFor(reg, promhttp.HandlerOpts{})

			listener, e := net.Listen("tcp", ":"+strconv.Itoa(int(port)))
			if e != nil {
				return e
			}

			fmt.Printf("listening for connections on port %d\n", port)
			return http.Serve(listener, handler)
		},
	}

	set := pflag.NewFlagSet("exporter", pflag.ExitOnError)

	set.Uint16Var(&port, "port", 9000, "Specify the port the exporter will listen on.")
	set.StringVar(&hubhost, "hub", "http://127.0.0.1:4444", "Specify the full hostname of the remote Grid Hub you want to export metrics from.")

	cmd.Flags().AddFlagSet(set)

	return cmd
}
