# Create k8s cluster with nodepool and run main services (server and ui nginx) on it.

## Doing manually

### Create a cluster

Generate personal access token https://www.digitalocean.com/docs/api/create-personal-access-token/. Export the token ```export API_TOKEN=_YOU_TOKEN_"

Chose master nodepool size and region from 
```bash
curl -X GET -H "Content-Type: application/json" -H "Authorization: Bearer ${API_TOKEN}" "https://api.digitalocean.com/v2/sizes" | jq .
```

Choose main nodepool size (eg `s-1vcpu-2gb`)

Choose main nodepool region (eg `nyc1`)

Choose cluster verion (eg `1.16.2-do.0`)

Create a cluster -
```bash
curl -X POST -H "Content-Type: application/json" -H "Authorization: Bearer ${API_TOKEN}" -d '{"name": "main-cluster","region": "nyc1","version": "1.16.2-do.0","tags": ["main"],"node_pools": [{"size": "s-1vcpu-2gb","count": 3,"name": "main-pool","tags": ["main"]}]}' "https://api.digitalocean.com/v2/kubernetes/clusters"
```

Response - 
```json
{
  "kubernetes_cluster": {
    "id": "9c97593c-5899-4ede-ae60-79cdab5c5127",
    "name": "main-cluster",
    "region": "nyc1",
    "version": "1.16.2-do.0",
    "cluster_subnet": "10.244.0.0/16",
    "service_subnet": "10.245.0.0/16",
    "vpc_uuid": "",
    "ipv4": "",
    "endpoint": "",
    "tags": [
      "main",
      "k8s",
      "k8s:9c97593c-5899-4ede-ae60-79cdab5c5127"
    ],
    "node_pools": [
      {
        "id": "7ed890ce-5150-411f-b42d-74d594cc546a",
        "name": "main-pool",
        "size": "s-1vcpu-2gb",
        "count": 3,
        "tags": [
          "main",
          "k8s",
          "k8s:9c97593c-5899-4ede-ae60-79cdab5c5127",
          "k8s:worker"
        ],
        "auto_scale": false,
        "min_nodes": 0,
        "max_nodes": 0,
        "nodes": [
          {
            "id": "3d3aa0ac-9ae1-4470-b669-3bad60afa018",
            "name": "",
            "status": {
              "state": "provisioning"
            },
            "droplet_id": "",
            "created_at": "2019-11-11T16:03:11Z",
            "updated_at": "2019-11-11T16:03:11Z"
          },
          {
            "id": "3ffe747d-21c3-4620-85a3-afacb35d5684",
            "name": "",
            "status": {
              "state": "provisioning"
            },
            "droplet_id": "",
            "created_at": "2019-11-11T16:03:11Z",
            "updated_at": "2019-11-11T16:03:11Z"
          },
          {
            "id": "41070831-421b-48f8-ac34-1258f9cf93eb",
            "name": "",
            "status": {
              "state": "provisioning"
            },
            "droplet_id": "",
            "created_at": "2019-11-11T16:03:11Z",
            "updated_at": "2019-11-11T16:03:11Z"
          }
        ]
      }
    ],
    "maintenance_policy": {
      "start_time": "20:00",
      "duration": "4h0m0s",
      "day": "any"
    },
    "auto_upgrade": false,
    "status": {
      "state": "provisioning",
      "message": "provisioning"
    },
    "created_at": "2019-11-11T16:03:11Z",
    "updated_at": "2019-11-11T16:03:11Z"
  }
}
```

### Run main

Get clusters kubeconfig - 
```bash
curl -X GET -H "Content-Type: application/json" -H "Authorization: Bearer ${API_TOKEN}" "https://api.digitalocean.com/v2/kubernetes/clusters/9c97593c-5899-4ede-ae60-79cdab5c5127/kubeconfig" > main-cluster-kubeconfig.yaml
```

Set kubeconfig env var
```bash
export KUBECONFIG=./main-cluster-kubeconfig.yaml
```

Create server and ui deployments and services

```bash
kubectl create -f server.yaml
```

## Rust code that does the same


TESTING:
export API_TOKEN=0df672ff8f3d5b20911c0566bc4a78248c633eadfd88b0f3d8420bbc33593df7

cluster-id = a9db436e-3a64-4ed8-9cb0-33f7eb924eaf
https://www.digitalocean.com/docs/kubernetes/how-to/configure-autoscaling/