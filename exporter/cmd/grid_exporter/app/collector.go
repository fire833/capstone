package app

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/valyala/fasthttp"
)

type GridCollector struct {
	hubhost string

	hubUp    prometheus.Gauge
	deSerErr prometheus.Gauge

	ready prometheus.Gauge

	deserialized *StatusMessageWrap
}

func NewGridCollector(hub string) *GridCollector {
	return &GridCollector{
		hubhost: hub,
		hubUp: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: "selenium_grid",
			Subsystem: "",
			Name:      "accesible",
			Help:      "This metric will be set to 1 if the last ping to the hub was successful, and 0 otherwise",
		}),
		deSerErr: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: "selenium_grid",
			Subsystem: "",
			Name:      "deserialization_error",
			Help:      "This metric will be set to 1 if there was an error deserializing the last status response from the server, and 0 otherwise",
		}),
		ready: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: "selenium_grid",
			Subsystem: "",
			Name:      "ready",
			Help:      "This metric will be set to 1 if the hub server indicates it is ready to receive requests, and 0 otherwise",
		}),

		deserialized: &StatusMessageWrap{},
	}
}

func (c *GridCollector) Describe(ch chan<- *prometheus.Desc) {

}

func (c *GridCollector) Collect(ch chan<- prometheus.Metric) {
	_, res, e := fasthttp.Get([]byte{}, c.hubhost+"/status")
	if e != nil {
		fmt.Println(e)
		c.hubUp.Set(0)
		ch <- c.hubUp
	} else {
		c.hubUp.Set(1)
		ch <- c.hubUp

		if e := json.Unmarshal(res, c.deserialized); e != nil {
			fmt.Println(e)
			c.deSerErr.Set(1)
			ch <- c.deSerErr
		} else {
			c.deSerErr.Set(0)
			ch <- c.deSerErr
		}

		if c.deserialized.Value.Ready {
			c.ready.Set(1)
			ch <- c.ready
		} else {
			c.ready.Set(0)
			ch <- c.ready
		}
	}

}

// Grid status schema

type StatusMessageWrap struct {
	Value StatusMessage `json:"value"`
}

type StatusMessage struct {
	Ready   bool         `json:"ready"`
	Message string       `json:"message"`
	Nodes   []NodeStatus `json:"nodes"`
}

type NodeStatus struct {
	ID          string `json:"id"`
	URI         string `json:"uri"`
	MaxSessions int    `json:"maxSessions"`
	OsInfo      struct {
		Arch    string `json:"arch"`
		Name    string `json:"name"`
		Version string `json:"version"`
	} `json:"osInfo"`
	HeartbeatPeriod int    `json:"heartbeatPeriod"`
	Availability    string `json:"availability"`
	Version         string `json:"version"`
	Slots           []struct {
		ID struct {
			HostID string `json:"hostId"`
			ID     string `json:"id"`
		} `json:"id"`
		LastStarted time.Time   `json:"lastStarted"`
		Session     interface{} `json:"session"`
		Stereotype  struct {
			BrowserName  string `json:"browserName"`
			PlatformName string `json:"platformName"`
		} `json:"stereotype"`
	} `json:"slots"`
}
