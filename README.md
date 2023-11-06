# Benchmarking webrtc-rs

## Setup

* GCP Compute-Shell -> client
* AWS ec2 instance -> server

## With data channels

```
Throughput: 0 MBits/s, 515 pkts, 515 loops
Throughput: 0 MBits/s, 559 pkts, 559 loops
Throughput: 0 MBits/s, 554 pkts, 554 loops
Throughput: 0 MBits/s, 540 pkts, 540 loops
```

## With plain UDP socket

```
Throughput: 880 MBits/s, 93771 pkts, 93771 loops
Throughput: 1152 MBits/s, 122477 pkts, 122477 loops
Throughput: 952 MBits/s, 101799 pkts, 101799 loops
Throughput: 952 MBits/s, 101759 pkts, 101759 loops
Throughput: 1096 MBits/s, 116284 pkts, 116284 loops
Throughput: 1176 MBits/s, 125489 pkts, 125489 loops
```

## W/O SCTP and DTLS

```
Throughput: 896 MBits/s, 95532 pkts, 95532 loops
Throughput: 952 MBits/s, 101378 pkts, 101378 loops
Throughput: 976 MBits/s, 103737 pkts, 103737 loops
Throughput: 984 MBits/s, 104559 pkts, 104559 loops
Throughput: 1144 MBits/s, 121860 pkts, 121860 loops
Throughput: 1064 MBits/s, 113120 pkts, 113120 loops
Throughput: 1096 MBits/s, 116358 pkts, 116358 loops
```
