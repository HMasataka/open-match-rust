# Open Match Rust

open-match implementation written in Rust

## Dependency

```bash
kubectl apply --namespace open-match \
    -f https://open-match.dev/install/v1.8.0/yaml/01-open-match-core.yaml
```

```bash
kubectl apply --namespace open-match \
  -f https://open-match.dev/install/v1.8.0/yaml/06-open-match-override-configmap.yaml \
  -f https://open-match.dev/install/v1.8.0/yaml/07-open-match-default-evaluator.yaml
```

### Uninstall

```bash
kubectl delete namespace open-match
```

## Dev

```bash
skaffold dev --port-forward
```
