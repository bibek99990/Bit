import discord
from discord.ext import commands
import yt_dlp
import os
import asyncio
from dotenv import load_dotenv
from flask import Flask
from threading import Thread

# Load .env
load_dotenv()
TOKEN = os.getenv("DISCORD_TOKEN")

# Discord setup
intents = discord.Intents.default()
intents.message_content = True
bot = commands.Bot(command_prefix="!", intents=intents)

progress_message = None

# Flask server (keep alive)
app = Flask('')

@app.route('/')
def home():
    return "Bot is running!"

def run():
    app.run(host='0.0.0.0', port=8080)

def keep_alive():
    t = Thread(target=run)
    t.start()

# Download progress hook
def progress_hook(d):
    global progress_message

    if d['status'] == 'downloading':
        percent = d.get('_percent_str', '0%')
        speed = d.get('_speed_str', '0')
        eta = d.get('_eta_str', '0')

        text = f"📥 Downloading...\nProgress: {percent}\nSpeed: {speed}\nETA: {eta}"

        if progress_message:
            asyncio.run_coroutine_threadsafe(
                progress_message.edit(content=text),
                bot.loop
            )

@bot.event
async def on_ready():
    print(f"✅ Bot Online: {bot.user}")

@bot.command()
async def audio(ctx, url):
    global progress_message

    progress_message = await ctx.send("⏳ Starting download...")

    ydl_opts = {
        "format": "bestaudio/best",
        "outtmpl": "audio.%(ext)s",
        "noplaylist": True,
        "progress_hooks": [progress_hook],
        "postprocessors": [{
            "key": "FFmpegExtractAudio",
            "preferredcodec": "mp3",
            "preferredquality": "192"
        }]
    }

    try:
        loop = asyncio.get_event_loop()

        await loop.run_in_executor(
            None,
            lambda: yt_dlp.YoutubeDL(ydl_opts).download([url])
        )

        await ctx.send("✅ Uploading audio...")
        await ctx.send(file=discord.File("audio.mp3"))

        os.remove("audio.mp3")

    except Exception as e:
        await ctx.send(f"❌ Error: {e}")

# Start Flask keep alive
keep_alive()

# Run bot
bot.run(TOKEN)
