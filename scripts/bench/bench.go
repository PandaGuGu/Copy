// bench — zero-dependency HTTP load tester for local capacity baselines.
//
// Usage:
//
//	go run ./scripts/bench/bench.go -url http://127.0.0.1:8080/api/v1/videos -n 2000 -c 50
//
// Output: total time, QPS, avg / p50 / p95 / p99 latency, status-code histogram.
package main

import (
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

type result struct {
	status int
	dur    time.Duration
}

func main() {
	url := flag.String("url", "http://127.0.0.1:8080/api/v1/videos", "target URL")
	total := flag.Int("n", 1000, "total requests")
	conc := flag.Int("c", 20, "concurrency")
	timeout := flag.Duration("timeout", 10*time.Second, "per-request timeout")
	flag.Parse()

	if *total <= 0 || *conc <= 0 {
		fmt.Fprintln(os.Stderr, "n and c must be positive")
		os.Exit(2)
	}

	client := &http.Client{Timeout: *timeout}
	results := make(chan result, *total)
	var sent atomic.Int64
	var failed atomic.Int64

	start := time.Now()

	var wg sync.WaitGroup
	worker := func() {
		defer wg.Done()
		for {
			n := sent.Add(1)
			if int(n) > *total {
				return
			}
			t0 := time.Now()
			resp, err := client.Get(*url)
			if err != nil {
				failed.Add(1)
				results <- result{status: 0, dur: time.Since(t0)}
				continue
			}
			_, _ = io.Copy(io.Discard, resp.Body)
			_ = resp.Body.Close()
			results <- result{status: resp.StatusCode, dur: time.Since(t0)}
		}
	}

	for i := 0; i < *conc; i++ {
		wg.Add(1)
		go worker()
	}
	wg.Wait()
	close(results)
	elapsed := time.Since(start)

	durs := make([]time.Duration, 0, *total)
	statusCount := map[int]int{}
	for r := range results {
		durs = append(durs, r.dur)
		statusCount[r.status]++
	}
	sort.Slice(durs, func(i, j int) bool { return durs[i] < durs[j] })

	percentile := func(p float64) time.Duration {
		if len(durs) == 0 {
			return 0
		}
		idx := int(p * float64(len(durs)))
		if idx >= len(durs) {
			idx = len(durs) - 1
		}
		return durs[idx]
	}
	var sum time.Duration
	for _, d := range durs {
		sum += d
	}

	fmt.Printf("=== bench result ===\n")
	fmt.Printf("target      : %s\n", *url)
	fmt.Printf("requests    : %d (conc=%d)\n", *total, *conc)
	fmt.Printf("elapsed     : %.2fs\n", elapsed.Seconds())
	fmt.Printf("qps         : %.1f\n", float64(*total)/elapsed.Seconds())
	if len(durs) > 0 {
		fmt.Printf("avg latency : %s\n", sum/time.Duration(len(durs)))
		fmt.Printf("p50         : %s\n", percentile(0.50))
		fmt.Printf("p95         : %s\n", percentile(0.95))
		fmt.Printf("p99         : %s\n", percentile(0.99))
	}
	fmt.Printf("errors      : %d\n", failed.Load())
	fmt.Printf("status codes:\n")
	for code, n := range statusCount {
		fmt.Printf("  %d -> %d\n", code, n)
	}
}
