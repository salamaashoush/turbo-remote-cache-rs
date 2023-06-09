# turbo-repo-remote-cache-rs
[![CircleCI](https://dl.circleci.com/status-badge/img/gh/salamaashoush/turbo-remote-cache-rs/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/salamaashoush/turbo-remote-cache-rs/tree/main)

Fast turbo remote cache server written in Rust.

if you are using turbo and you want to have a self-hosted remote cache server this is for you.

1. It is fast ⚡️.
2. It supports multiple object storage providers (S3, GCP, Azure, and file).


## Usage

### Binary
1. Download the latest binary from [release page](https://github.com/salamaashoush/turbo-remote-cache-rs/releases)
2. Copy `.env.example` to same directory as the binary and rename it to `.env`.
3. Edit `.env` file to your needs (`TURBO_TOKENS`, `BUCKET_NAME` are required) please refer to [`.env.example`](./.env.example) for more information about required vars for each cloud storage provider.
4. Run the binary.
```bash
./turbo-repo-remote-cache-rs
```
5. Provide the url to turbo cli
```bash
turbo build --api="http://localhost:4000" --token=<token> --team=<team> 
```
you can also set `teamId` and `apiUrl` `.turbo/config.json` in the root of your project.

6. Enjoy your self-hosted remote cache and faster builds 🚀.

### Docker
1. Create a docker file.
```Dockerfile
FROM salamaashoush/turbo-remote-cache-rs
ENV PORT=4000
ENV FS_PATH=/tmp
ENV BUCKET_NAME=cache
ENV TURBO_TOKENS="token1,token2,token3"
EXPOSE 4000
CMD ["turbo-remote-cache"]
```
2. Build the image.
```bash
docker build -t your-image .
```
3. Run the image.
```bash
docker run -p 4000:4000 -v ./cache:/tmp/cache your-image
```
4. Provide the URL to turbo cli
```bash
turbo build --api="http://localhost:4000" --token=<token> --team=<team> 
```
you can also set `teamId` and `apiUrl` `.turbo/config.json` in the root of your project.

## Kubernetes
See example in [examples/k8s](./examples/k8s), Don't forget to change the spec and env vars for your needs before applying it (NOTE that it is just an example and it is not production ready).

## Environment Variables

### Required
| Name | Description | Default |
|------|-------------|---------|
| `TURBO_TOKENS` | Comma separated list of turbo tokens that are allowed to access the cache. | `""` |
| `BUCKET_NAME` | Name of the bucket to store the cache in. | `"cache"` |
| `STORAGE_PROVIDER` | Storage provider to use. `s3`, `azure`, `gcs` or `file` | `"file"` |

### File Storage Provider
| Name | Description | Default |
|------|-------------|---------|
| `FS_PATH` | Path to store the cache in. | os temp dir |

### S3 Storage Provider
| Name | Description | Default |
|------|-------------|---------|
| `AWS_ACCESS_KEY_ID` | AWS access key id. | `""` |
| `AWS_SECRET_ACCESS_KEY` | AWS secret access key. | `""` |
| `AWS_DEFAULT_REGION` | AWS region. | `""` |
| `AWS_ENDPOINT` | AWS endpoint. | `""` |
| `AWS_SESSION_TOKEN` | AWS session token | `""` |
| `AWS_CONTAINER_CREDENTIALS_RELATIVE_URI` |[AWS bucket endpoint.](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-iam-roles.html) | `""` |
| `AWS_ALLOW_HTTP` | set to “true” to permit HTTP connections without TLS. | `false` |
| `AWS_PROFILE` | set profile name, requires aws_profile feature enabled | `""` |

### Azure Storage Provider
| Name | Description | Default |
|------|-------------|---------|
| `AZURE_STORAGE_ACCOUNT_NAME` | storage account name. | `""` |
| `AZURE_STORAGE_ACCOUNT_KEY` | storage account master key | `""` |
| `AZURE_STORAGE_ACCESS_KEY` | alias for `AZURE_STORAGE_ACCOUNT_KEY` | `""` |
| `AZURE_STORAGE_CLIENT_ID` | client id for service principal authorization | `""` |
| `AZURE_STORAGE_CLIENT_SECRET` |  client secret for service principal authorization | `""` |
| `AZURE_STORAGE_TENANT_ID` | tenant id used in oauth flows | `""` |

### GCS Storage Provider
| Name | Description | Default |
|------|-------------|---------|
| `GOOGLE_SERVICE_ACCOUNT` | location of service account file | `""` |
| `GOOGLE_SERVICE_ACCOUNT_PATH` | (alias) location of service account file | `""` |
| `SERVICE_ACCOUNT` | (alias) location of service account file | `""` |
| `GOOGLE_SERVICE_ACCOUNT_KEY` | JSON serialized service account key | `""` |
| `GOOGLE_BUCKET` | bucket name | `""` |
| `GOOGLE_BUCKET_NAME` | (alias) bucket name | `""` |

## Todo

- [ ] Support turbo headers `x-artifact-duration`, `x-artifact-tag`, `x-artifact-client-ci` and `x-artifact-client-interactive` right now those are ignored and they don't affect the cache.
- [ ] Add nx cloud support.
- [ ] Add more advanced authentication support.
- [ ] Maybe having a dashboard to manage teams and projects would be nice.

