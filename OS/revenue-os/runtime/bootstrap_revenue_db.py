#!/usr/bin/env python3
# Small SQLite demonstration store for Revenue OS—not statutory accounting.
from pathlib import Path
import sqlite3
DB=Path(__file__).with_name('data')/'revenue_demo.sqlite3'
DDL='''
pragma foreign_keys=on;
create table if not exists customer(id text primary key,name text not null,status text not null,created_at text not null);
create table if not exists opportunity(id text primary key,customer_id text not null references customer(id),stage text not null,amount real,currency text,next_step text,updated_at text not null);
create table if not exists document(id text primary key,file_hash text unique not null,type text not null,status text not null,extracted_json text,created_at text not null);
create table if not exists invoice(id text primary key,customer_id text not null references customer(id),number text not null,amount real not null,currency text not null,due_date text not null,status text not null,unique(customer_id,number));
create table if not exists audit_event(id integer primary key autoincrement,event_type text not null,resource_id text,payload_json text not null,created_at text not null);
'''
DB.parent.mkdir(parents=True,exist_ok=True)
with sqlite3.connect(DB) as cx: cx.executescript(DDL)
print(DB)
