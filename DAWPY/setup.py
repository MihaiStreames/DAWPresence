import os

from setuptools import find_packages, setup


setup(
    name="dawpresence",
    version="1.0.0",
    author="MihaiStreames",
    description="DAW Discord Rich Presence - Show what you're making on your DAW in Discord",
    url="https://github.com/MihaiStreames/DAWPresence",
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
        "loguru>=0.7.3",
    ],
    extras_require={
        "windows": ["pywin32>=311"],
    },
    entry_points={
        "console_scripts": [
            "dawpresence=main:main",
        ],
    },
    include_package_data=True,
    package_data={
        "": ["*.json", "*.ico", "*.png", "*.md", "*.txt"],
        "config": ["*.json"],
        "assets": ["*.ico", "*.png"],
    },
    data_files=[
        ("config", ["config/daws.json"]),
        (
            "assets",
            (["assets/red.ico", "assets/green.ico"] if os.path.exists("../assets") else []),
        ),
    ],
)
