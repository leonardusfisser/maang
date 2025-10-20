#!/usr/bin/env fish
# deploy.fish - Lillpepe deployment script for MP100 (Debian)
# No Docker, no Kubernetes, pure Rust + rsync

# ============================================================================
# CONFIGURATION
# ============================================================================

set SERVER "user@mp100"
set DEPLOY_PATH "/opt/lillpepe"
set BINARY_NAME "lillpepe"

# ============================================================================
# BUILD
# ============================================================================

function build_release
    echo "🔨 Building release binary..."
    cargo build --release --target x86_64-unknown-linux-gnu

    if test $status -ne 0
        echo "❌ Build failed"
        exit 1
    end

    echo "✓ Build successful"
end

# ============================================================================
# DEPLOY
# ============================================================================

function deploy_binary
    echo "📦 Deploying binary to $SERVER..."

    rsync -avz --progress \
        ./target/release/$BINARY_NAME \
        $SERVER:$DEPLOY_PATH/bin/

    if test $status -ne 0
        echo "❌ Binary deployment failed"
        exit 1
    end

    echo "✓ Binary deployed"
end

function deploy_templates
    echo "📄 Deploying templates..."

    rsync -avz --progress --delete \
        ./templates/ \
        $SERVER:$DEPLOY_PATH/templates/

    if test $status -ne 0
        echo "❌ Template deployment failed"
        exit 1
    end

    echo "✓ Templates deployed"
end

function deploy_static
    echo "🎨 Deploying static assets..."

    rsync -avz --progress --delete \
        ./static/ \
        $SERVER:$DEPLOY_PATH/static/

    if test $status -ne 0
        echo "❌ Static assets deployment failed"
        exit 1
    end

    echo "✓ Static assets deployed"
end

function deploy_migrations
    echo "🗄️  Deploying database migrations..."

    rsync -avz --progress \
        ./migrations/ \
        $SERVER:$DEPLOY_PATH/migrations/

    if test $status -ne 0
        echo "❌ Migrations deployment failed"
        exit 1
    end

    echo "✓ Migrations deployed"
end

# ============================================================================
# DATABASE
# ============================================================================

function run_migrations
    echo "🗄️  Running database migrations..."

    ssh $SERVER "cd $DEPLOY_PATH && \
        surreal import \
            --conn http://localhost:8000 \
            --user root \
            --pass root \
            --ns lillpepe \
            --db main \
            migrations/001_initial_schema.surql"

    if test $status -ne 0
        echo "⚠️  Migration may have already run"
    else
        echo "✓ Migrations completed"
    end
end

# ============================================================================
# SERVICE MANAGEMENT
# ============================================================================

function restart_service
    echo "🔄 Restarting service..."

    ssh $SERVER "sudo systemctl restart lillpepe"

    if test $status -ne 0
        echo "❌ Service restart failed"
        exit 1
    end

    sleep 2

    ssh $SERVER "sudo systemctl status lillpepe --no-pager"

    echo "✓ Service restarted"
end

function check_health
    echo "🏥 Checking service health..."

    sleep 3

    set response (curl -s http://mp100:8080/api/v1/health)

    if string match -q "*healthy*" $response
        echo "✓ Service is healthy"
        echo $response | jq .
    else
        echo "❌ Service health check failed"
        echo $response
        exit 1
    end
end

# ============================================================================
# BACKUP
# ============================================================================

function backup_before_deploy
    echo "💾 Creating backup..."

    set timestamp (date +%Y%m%d_%H%M%S)
    set backup_name "lillpepe_backup_$timestamp"

    ssh $SERVER "mkdir -p $DEPLOY_PATH/backups && \
        cp $DEPLOY_PATH/bin/$BINARY_NAME $DEPLOY_PATH/backups/$BINARY_NAME.$timestamp 2>/dev/null || true"

    echo "✓ Backup created"
end

# ============================================================================
# ROLLBACK
# ============================================================================

function rollback
    echo "⏪ Rolling back to previous version..."

    ssh $SERVER "cd $DEPLOY_PATH/backups && \
        ls -t $BINARY_NAME.* | head -1 | xargs -I {} cp {} $DEPLOY_PATH/bin/$BINARY_NAME"

    if test $status -ne 0
        echo "❌ Rollback failed"
        exit 1
    end

    restart_service
    echo "✓ Rolled back successfully"
end

# ============================================================================
# MAIN DEPLOYMENT FLOW
# ============================================================================

function deploy
    echo "🚀 Starting deployment to $SERVER"
    echo "=================================="

    # Build
    build_release

    # Backup
    backup_before_deploy

    # Deploy
    deploy_binary
    deploy_templates
    deploy_static
    deploy_migrations

    # Database
    run_migrations

    # Restart
    restart_service

    # Verify
    check_health

    echo ""
    echo "=================================="
    echo "✅ Deployment completed successfully!"
    echo "🌐 Admin: http://mp100:8080/admin"
    echo "📊 API: http://mp100:8080/api/v1/health"
end

# ============================================================================
# INITIAL SETUP (first time only)
# ============================================================================

function initial_setup
    echo "🏗️  Running initial setup on $SERVER..."

    # Create directories
    ssh $SERVER "sudo mkdir -p $DEPLOY_PATH/{bin,templates,static,migrations,logs,backups,tenants} && \
        sudo chown -R lillpepe:lillpepe $DEPLOY_PATH"

    # Create systemd service
    ssh $SERVER "sudo tee /etc/systemd/system/lillpepe.service > /dev/null << 'EOF'
[Unit]
Description=Lillpepe White Label SaaS Platform
After=network.target surrealdb.service

[Service]
Type=simple
User=lillpepe
WorkingDirectory=$DEPLOY_PATH
Environment=\"HOST=0.0.0.0\"
Environment=\"PORT=8080\"
Environment=\"DATABASE_URL=127.0.0.1:8000\"
Environment=\"DATABASE_NAMESPACE=lillpepe\"
Environment=\"DATABASE_NAME=main\"
Environment=\"RUST_LOG=info\"
ExecStart=$DEPLOY_PATH/bin/$BINARY_NAME
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF"

    # Enable service
    ssh $SERVER "sudo systemctl daemon-reload && \
        sudo systemctl enable lillpepe"

    echo "✓ Initial setup completed"
end

# ============================================================================
# MONITORING
# ============================================================================

function show_logs
    echo "📋 Showing service logs..."
    ssh $SERVER "sudo journalctl -u lillpepe -f --no-pager"
end

function show_status
    echo "📊 Service status:"
    ssh $SERVER "sudo systemctl status lillpepe --no-pager"
end

# ============================================================================
# USAGE
# ============================================================================

function show_usage
    echo "Lillpepe Deployment Tool"
    echo ""
    echo "Usage: ./deploy.fish [command]"
    echo ""
    echo "Commands:"
    echo "  deploy          - Full deployment (build + deploy + restart)"
    echo "  build           - Build release binary only"
    echo "  setup           - Initial server setup (first time)"
    echo "  rollback        - Rollback to previous version"
    echo "  restart         - Restart service"
    echo "  logs            - Show live logs"
    echo "  status          - Show service status"
    echo "  health          - Check service health"
    echo ""
end

# ============================================================================
# COMMAND DISPATCHER
# ============================================================================

if test (count $argv) -eq 0
    deploy
else
    switch $argv[1]
        case deploy
            deploy
        case build
            build_release
        case setup
            initial_setup
        case rollback
            rollback
        case restart
            restart_service
        case logs
            show_logs
        case status
            show_status
        case health
            check_health
        case '*'
            show_usage
    end
end