"""Main application setup — post_init, cleanup, and main()."""

import asyncio
import os
import signal
import subprocess
import time

from aisb.config import (
    TOKEN, CLOUD_MODE, CLOUD_ORG_ID, MAX_TURNS, GROUP_CHAT_ID,
    PROJECTS_CONFIG, CHAT_ID,
    logger, Application, BotCommand,
    CommandHandler, CallbackQueryHandler, MessageHandler, MessageReactionHandler, filters,
    openai_client,
)
from aisb.sessions import user_sessions, init_sessions
from aisb.commands import cmd_start, cmd_status, cmd_agents, cmd_project, cmd_new, cmd_stop, cmd_skills, cmd_kill, cmd_next, cmd_billing, cmd_menu, cmd_restart, cmd_autonomous, cmd_cost, cmd_graph, cmd_purge, cmd_cleanup
from aisb.account import cmd_account, handle_account_callback, handle_usage_callback, handle_dispatch_callback
from aisb.aisb_analysis import handle_aisb_callback
from aisb.routines import cmd_routine
from aisb.setup_wizard import cmd_setup, check_first_run, cmd_addproject, cmd_newproject
--
async def post_init(app: Application):
    commands = [
        BotCommand("start", "Info & quick reference"),
        BotCommand("skills", "All Claude Code commands"),
        BotCommand("project", "Switch project"),
        BotCommand("new", "Reset session"),
        BotCommand("stop", "Stop running session"),
        BotCommand("kill", "Kill all sessions + clean"),
        BotCommand("cleanup", "Full disk cleanup (deep · all-in)"),
        BotCommand("restart", "Restart the bot via systemd"),
        BotCommand("next", "Reconnect all sessions after /login"),
        BotCommand("billing", "Claude Code usage + switch account"),
        BotCommand("status", "System status"),
        BotCommand("agents", "Agent list"),
        # AISB Brain
        BotCommand("aisb", "AI Super Brain — smart routing"),
        BotCommand("aisb_status", "AISB dashboard (ZION)"),
        BotCommand("aisb_audit", "Ecosystem health check"),
