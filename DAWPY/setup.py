import os

from setuptools import setup, find_packages


def get_version():
    """Extract version from main.py"""
    with open("main.py", "r") as f:
        for line in f:
            if "APP_VERSION" in line:
                return line.split('"')[1]
    return "2.0.0"


def get_long_description():
    """Get long description from README"""
    if os.path.exists("README.md"):
        with open("README.md", "r", encoding="utf-8") as f:
            return f.read()
    return "DAW Discord Rich Presence - Show your DAW activity in Discord"


setup(
    name="dawrpc",
    version=get_version(),
    author="MihaiStreames",
    description="DAW Discord Rich Presence - Show what you're making on your DAW in Discord",
    long_description=get_long_description(),
    long_description_content_type="text/markdown",
    url="https://github.com/MihaiStreames/DAWRPC",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Environment :: Win32 (MS Windows)",
        "Intended Audience :: End Users/Desktop",
        "License :: OSI Approved :: MIT License",
        "Operating System :: Microsoft :: Windows",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Communications :: Chat",
        "Topic :: Multimedia :: Sound/Audio",
        "Topic :: System :: Monitoring",
    ],
    python_requires=">=3.7",
    install_requires=[
        "PyQt5>=5.15.11",
        "pypresence>=4.3.0",
        "psutil>=7.1.0",
    ],
    extras_require={
        'windows': ['pywin32>=311'],
    },
    entry_points={
        'console_scripts': [
            'dawrpc=main:main',
        ],
    },
    include_package_data=True,
    package_data={
        '': ['*.json', '*.ico', '*.png', '*.md', '*.txt'],
        'config': ['*.json'],
        'assets': ['*.ico', '*.png'],
    },
    data_files=[
        ('config', ['config/daws.json']),
        ('assets', ['assets/red.ico', 'assets/green.ico'] if os.path.exists('assets') else []),
    ],
)
