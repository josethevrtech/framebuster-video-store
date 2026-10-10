FRAME_HOST=${FRAME_HOST:-steamos@10.0.0.25}
FRAME_SSH_KEY=${FRAME_SSH_KEY:-$HOME/.config/steamos-devkit/devkit_rsa}
MATINEEVR_GAME_ID=MatineeVR

frame_ssh() {
    ssh -o BatchMode=yes -o ConnectTimeout=10 -i "$FRAME_SSH_KEY" "$FRAME_HOST" "$@"
}

frame_scp() {
    scp -q -o BatchMode=yes -o ConnectTimeout=10 -i "$FRAME_SSH_KEY" "$@"
}
