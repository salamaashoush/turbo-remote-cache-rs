# turbo-repo-remote-cache-rs

Fast turbo remote cache server written in Rust.
if you are using turbo and you want to have a self hosted remote cache server this is for you.

1. It is fast ⚡️.
2. It supports multiple object storage provider (s3, gcp, azure, file).
  

## Usage

1. Download the latest binary from [release page](https://github.com/salamaashoush/turbo-remote-cache-rs/releases)
2. Copy `.env.example` to same directory as the binary and rename it to `.env`.
3. Edit `.env` file to your needs (`TURBO_TOKENS`, `BUCKET_NAME` are required) please refer to [`.env.example`](./.env.example) for more information about required vars for each cloud storage provider.
4. Run the binary.
```bash
./turbo-repo-remote-cache-rs
```
1. Provide the url to turbo cli
```bash
turbo build --api="http://localhost:4000" --token=<token> --team=<team> 
```
you can also set `teamId` and `apiUrl` `.turbo/config.json` in the root of your project.

1. Enjoy your self hosted remote cache and faster builds 🚀.

## Todo

- [ ] Support turbo headers `x-artifact-duration`, `x-artifact-tag`, `x-artifact-client-ci` and `x-artifact-client-interactive` right now those are ignored and they don't affect the cache.
- [ ] Add nx cloud support.
- [ ] Publish docker images.
- [ ] Add more advanced authentication support.
- [ ] Maybe having a dashboard to manage teams and projects would be nice.

