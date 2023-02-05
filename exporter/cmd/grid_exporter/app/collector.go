package app

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/valyala/fasthttp"
)

const gridNs string = "selenium_grid"

type GridCollector struct {
	hubhost string

	hubUp                    prometheus.Gauge
	deSerErr                 prometheus.Gauge
	numNodes                 prometheus.Gauge
	numUsedSessionsAggregate prometheus.Gauge
	maxSessionsAggregate     prometheus.Gauge
	queueSize                prometheus.Gauge
	queueDeSerErr            prometheus.Gauge

	ready prometheus.Gauge

	deserialized      *StatusMessageWrap
	deserializedQueue *QueueStatusWrap
}

func NewGridCollector(hub string) *GridCollector {
	return &GridCollector{
		hubhost: hub,

		hubUp: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "accessible",
			Help:      "This metric will be set to 1 if the last ping to the hub was successful, and 0 otherwise",
		}),

		deSerErr: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "deserialization_error",
			Help:      "This metric will be set to 1 if there was an error deserializing the last status response from the server, and 0 otherwise",
		}),

		queueDeSerErr: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "queue_deserialization_error",
			Help:      "This metric will be set to 1 if there was an error deserializing the last queue status response from the server, and 0 otherwise",
		}),

		ready: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "ready",
			Help:      "This metric will be set to 1 if the hub server indicates it is ready to receive requests, and 0 otherwise",
		}),

		numNodes: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "num_nodes",
			Help:      "This metric provides the current number of nodes within the Selenium Grid cluster",
		}),

		numUsedSessionsAggregate: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "num_sessions_aggregated",
			Help:      "This metric provides an aggregated quantity of the number of sessions running within this Selenium Grid cluster",
		}),

		maxSessionsAggregate: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "max_sessions_aggregated",
			Help:      "This metric provides an aggregated quantity of the maximum number of sessions able to be run within this Selenium Grid cluster",
		}),

		queueSize: prometheus.NewGauge(prometheus.GaugeOpts{
			Namespace: gridNs,
			Subsystem: "",
			Name:      "queue_size",
			Help:      "This metric provides information on the queue size within your Selenium Grid Hub",
		}),

		deserialized:      &StatusMessageWrap{},
		deserializedQueue: &QueueStatusWrap{},
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
			// fmt.Println(e)
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

		maxSessions := 0
		currSessions := 0

		for _, node := range c.deserialized.Value.Nodes {
			maxSessions += node.MaxSessions

			for _, slot := range node.Slots {
				if slot.Session != nil {
					currSessions++
				}
			}
		}

		// Set number of nodes.
		c.numNodes.Set(float64(len(c.deserialized.Value.Nodes)))
		ch <- c.numNodes
		// Set the max sessions
		c.maxSessionsAggregate.Set(float64(maxSessions))
		ch <- c.maxSessionsAggregate
		// Set the current utilized sessions
		c.numUsedSessionsAggregate.Set(float64(currSessions))
		ch <- c.numUsedSessionsAggregate

	}

	if _, res, e1 := fasthttp.Get([]byte{}, c.hubhost+"/se/grid/newsessionqueue/queue"); e1 == nil {
		if e := json.Unmarshal(res, c.deserializedQueue); e != nil {
			c.queueDeSerErr.Set(1)
			ch <- c.queueDeSerErr
		} else {
			c.queueDeSerErr.Set(0)
			ch <- c.queueDeSerErr
		}

		c.queueSize.Set(float64(len(c.deserializedQueue.Value)))
		ch <- c.queueSize
	}

}

// Grid status schema

type StatusMessageWrap struct {
	Value StatusMessage `json:"value"`
}

type StatusMessage struct {
	Ready   bool          `json:"ready"`
	Message string        `json:"message"`
	Nodes   []*NodeStatus `json:"nodes"`
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
	HeartbeatPeriod int         `json:"heartbeatPeriod"`
	Availability    string      `json:"availability"`
	Version         string      `json:"version"`
	Slots           []*NodeSlot `json:"slots"`
}

type NodeSlot struct {
	ID struct {
		HostID string `json:"hostId"`
		ID     string `json:"id"`
	} `json:"id"`
	LastStarted time.Time    `json:"lastStarted"`
	Session     *NodeSession `json:"session"`
	Stereotype  struct {
		BrowserName  string `json:"browserName"`
		PlatformName string `json:"platformName"`
	} `json:"stereotype"`
}

type NodeSession struct {
	Capabilities *NodeSessionCapabilities `json:"capabilities"`
	SessionID    string                   `json:"sessionId"`
	Start        time.Time                `json:"start"`
	Stereotype   struct {
		BrowserName    string `json:"browserName"`
		BrowserVersion string `json:"browserVersion"`
		PlatformName   string `json:"platformName"`
		SeNoVncPort    int    `json:"se:noVncPort"`
		SeVncEnabled   bool   `json:"se:vncEnabled"`
	} `json:"stereotype"`
	URI string `json:"uri"`
}

type NodeSessionCapabilities struct {
	AcceptInsecureCerts bool   `json:"acceptInsecureCerts"`
	BrowserName         string `json:"browserName"`
	BrowserVersion      string `json:"browserVersion"`
	Chrome              struct {
		ChromedriverVersion string `json:"chromedriverVersion"`
		UserDataDir         string `json:"userDataDir"`
	} `json:"chrome"`
	GoogChromeOptions struct {
		DebuggerAddress string `json:"debuggerAddress"`
	} `json:"goog:chromeOptions"`
	NetworkConnectionEnabled bool   `json:"networkConnectionEnabled"`
	PageLoadStrategy         string `json:"pageLoadStrategy"`
	PlatformName             string `json:"platformName"`
	Proxy                    struct {
	} `json:"proxy"`
	SeBidiEnabled             bool   `json:"se:bidiEnabled"`
	SeCdp                     string `json:"se:cdp"`
	SeCdpVersion              string `json:"se:cdpVersion"`
	SeVnc                     string `json:"se:vnc"`
	SeVncEnabled              bool   `json:"se:vncEnabled"`
	SeVncLocalAddress         string `json:"se:vncLocalAddress"`
	SetWindowRect             bool   `json:"setWindowRect"`
	StrictFileInteractability bool   `json:"strictFileInteractability"`
	Timeouts                  struct {
		Implicit int `json:"implicit"`
		PageLoad int `json:"pageLoad"`
		Script   int `json:"script"`
	} `json:"timeouts"`
	UnhandledPromptBehavior       string `json:"unhandledPromptBehavior"`
	WebauthnExtensionCredBlob     bool   `json:"webauthn:extension:credBlob"`
	WebauthnExtensionLargeBlob    bool   `json:"webauthn:extension:largeBlob"`
	WebauthnVirtualAuthenticators bool   `json:"webauthn:virtualAuthenticators"`
}

type QueueStatusWrap struct {
	Value []*QueueStatus `json:"value"`
}

type QueueStatus struct {
	Capabilities []struct {
		BrowserName string `json:"browserName"`
	} `json:"capabilities"`
	RequestID string `json:"requestId"`
}
