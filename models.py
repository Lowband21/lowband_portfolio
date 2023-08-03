from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate
from flask import Flask, send_from_directory

app = Flask(__name__, static_folder='public')
app.config['SQLALCHEMY_DATABASE_URI'] = 'mysql://mysql:Lysergic@localhost/portfolio'
db = SQLAlchemy(app)
migrate = Migrate(app, db)

class Project(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(80), nullable=False)
    description = db.Column(db.String(240), nullable=False)

class Skill(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(80), nullable=False)

class Bio(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    bio = db.Column(db.String(1000), nullable=False)

def create_db():
    with app.app_context():
        db.create_all()

def fill_db():

    with app.app_context():
        skills = ['Python', 'Java', 'C&C++', 'Rust', 'Haskell', 'Git', 'Regex', 'Markdown', 'Vim', 'SQL', 'NoSQL', 'Embedded Systems', 'Networking', 'Artificial Intelligence', 'Linux System Administration', 'Docker', 'Virtualization', 'Bash Scripting', 'Automation']
        
        for skill in skills:
            existing_skill = Skill.query.filter_by(name=skill).first()
            if not existing_skill:
                s = Skill(name=skill)
                db.session.add(s)
        
        bio_text = "A diligent Computer Science student from the University of Denver with a strong academic record. I have a profound knowledge and hands-on experience in various programming languages including Python, Java, C&C++, Rust, and Haskell. My expertise also extends to Linux system administration, Docker, Virtualization, and Bash Scripting."
        existing_bio = Bio.query.first()
        if not existing_bio:
            bio = Bio(bio=bio_text)
            db.session.add(bio)

        projects = [
            {
                'name': 'Socket Painter',
                'description': "A networked graphical drawing application utilizing sockets to operate on the server-client model, multi-threading, and Java JPanel."
            },
            {
                'name': 'Huffman Compression',
                'description': "An implementation of Huffman Compression in C."
            },
            {
                'name': 'Cache Simulator',
                'description': "Implementations of direct mapped, set associative, and fully associative caches."
            },
            {
                'name': 'Binary Math',
                'description': "Implementations of binary addition, subtraction, multiplication, and division."
            },
            {
                'name': 'MDPlanner',
                'description': "An application developed in high school using Python to automate aspects of my planning by utilizing my notes app API (Joplin at the time). This project taught me a great deal about working with APIs and strings."
            },
            {
                'name': 'Canvas Todo',
                'description': "A simple program tying together the Canvas API and Todoist API to sync my assignments between Canvas and Todoist as a cron job."
            },
            {
                'name': 'Registration Database',
                'description': "A Python application I developed using Selenium and MariaDB to scrape all course information from a University website and create a database of it."
            },
            {
                'name': 'Linux System Administration',
                'description': "I've been running Linux exclusively on my personal devices for over 4 years."
            },
            {
                'name': 'Server Administration',
                'description': "I operate a personal Linux server that I use to run Docker applications and Virtual Machines with dedicated GPU Passthrough."
            },
            {
                'name': '2048 Game',
                'description': "An implementation of the game 2048 plus a Q-learning AI agent."
            },
            {
                'name': 'MQTT CMD Internal Testing and Validation',
                'description': "A async program developed in Rust providing an interface to an embedded system over MQTT."
            }
        ]

        for project in projects:
            existing_project = Project.query.filter_by(name=project["name"]).first()
            if not existing_project:
                p = Project(name=project["name"], description=project["description"])
                db.session.add(p)

        db.session.commit()
