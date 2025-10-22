-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Tenants table
CREATE TABLE tenants (
                         id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                         name VARCHAR(255) NOT NULL,
                         domain VARCHAR(255) NOT NULL UNIQUE,
                         status VARCHAR(50) NOT NULL DEFAULT 'provisioning',
                         config JSONB NOT NULL DEFAULT '{}',
                         created_at BIGINT NOT NULL,
                         updated_at BIGINT NOT NULL
);

CREATE INDEX idx_tenants_domain ON tenants(domain);
CREATE INDEX idx_tenants_status ON tenants(status);

-- Users table
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                       tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                       email VARCHAR(255) NOT NULL,
                       password_hash VARCHAR(255) NOT NULL,
                       salt VARCHAR(255) NOT NULL,
                       role VARCHAR(50) NOT NULL DEFAULT 'user',
                       status VARCHAR(50) NOT NULL DEFAULT 'active',
                       created_at BIGINT NOT NULL,
                       updated_at BIGINT NOT NULL,
                       UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);

-- Products table
CREATE TABLE products (
                          id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                          tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                          name VARCHAR(255) NOT NULL,
                          description TEXT NOT NULL,
                          price BIGINT NOT NULL,
                          currency VARCHAR(10) NOT NULL DEFAULT 'USD',
                          status VARCHAR(50) NOT NULL DEFAULT 'active',
                          image_url TEXT,
                          metadata JSONB NOT NULL DEFAULT '{}',
                          created_at BIGINT NOT NULL,
                          updated_at BIGINT NOT NULL
);

CREATE INDEX idx_products_tenant_id ON products(tenant_id);
CREATE INDEX idx_products_status ON products(status);

-- Services table
CREATE TABLE services (
                          id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                          tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                          name VARCHAR(255) NOT NULL,
                          description TEXT NOT NULL,
                          status VARCHAR(50) NOT NULL DEFAULT 'active',
                          metadata JSONB NOT NULL DEFAULT '{}',
                          created_at BIGINT NOT NULL,
                          updated_at BIGINT NOT NULL
);

CREATE INDEX idx_services_tenant_id ON services(tenant_id);
CREATE INDEX idx_services_status ON services(status);

-- Media table
CREATE TABLE media (
                       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                       tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
                       name VARCHAR(255) NOT NULL,
                       media_type VARCHAR(50) NOT NULL,
                       url TEXT NOT NULL,
                       size_bytes BIGINT NOT NULL,
                       metadata JSONB NOT NULL DEFAULT '{}',
                       created_at BIGINT NOT NULL
);

CREATE INDEX idx_media_tenant_id ON media(tenant_id);
CREATE INDEX idx_media_type ON media(media_type);

-- Add comments
COMMENT ON TABLE tenants IS 'Multi-tenant isolation - each tenant has isolated data';
COMMENT ON TABLE users IS 'User accounts scoped to tenants';
COMMENT ON TABLE products IS 'Products managed by tenants';
COMMENT ON TABLE services IS 'Services offered by tenants';
COMMENT ON TABLE media IS 'Media files (images, videos, audio) uploaded by tenants';