-- this file is used to initialize the postgres database
-- user table
CREATE TABLE IF NOT EXISTS t_user (
    id BIGINT PRIMARY KEY,
    avatar VARCHAR(128) NOT NULL DEFAULT '',
    username VARCHAR(64) NOT NULL,
    passwd VARCHAR(128) NOT NULL,
    email VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE t_user IS '用户表';
COMMENT ON COLUMN t_user.id IS '用户ID';
COMMENT ON COLUMN t_user.avatar IS '头像';
COMMENT ON COLUMN t_user.username IS '用户名（显示用）';
COMMENT ON COLUMN t_user.passwd IS '密码';
COMMENT ON COLUMN t_user.email IS '邮箱';
COMMENT ON COLUMN t_user.created_at IS '创建时间';
COMMENT ON COLUMN t_user.updated_at IS '更新时间';

-- chat type enum: single / group / private_channel / public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- chat table
CREATE TABLE IF NOT EXISTS t_chat (
    id BIGINT PRIMARY KEY,
    name VARCHAR(128) NOT NULL DEFAULT '',
    type chat_type NOT NULL DEFAULT 'single',
    members BIGINT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE t_chat IS '聊天信息表';
COMMENT ON COLUMN t_chat.id IS '聊天ID';
COMMENT ON COLUMN t_chat.name IS '聊天名称';
COMMENT ON COLUMN t_chat.type IS '聊天类型';
COMMENT ON COLUMN t_chat.members IS '聊天成员';
COMMENT ON COLUMN t_chat.created_at IS '创建时间';
COMMENT ON COLUMN t_chat.updated_at IS '更新时间';

-- message table
CREATE TABLE IF NOT EXISTS t_message (
    id BIGINT PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    images TEXT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE t_message IS '消息表';
COMMENT ON COLUMN t_message.id IS '消息ID';
COMMENT ON COLUMN t_message.chat_id IS '聊天ID';
COMMENT ON COLUMN t_message.sender_id IS '发送者ID';
COMMENT ON COLUMN t_message.content IS '消息内容';
COMMENT ON COLUMN t_message.images IS '消息图片';
COMMENT ON COLUMN t_message.created_at IS '创建时间';

-- create index for message table on chat_id and created_at order by created_at desc
CREATE INDEX idx_chat_id_created_at ON t_message (chat_id, created_at DESC);

-- create index for message table on sender_id and created_at order by created_at desc
CREATE INDEX idx_sender_id_created_at ON t_message (sender_id, created_at DESC);
