from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="dawrpc",
    version="1.0.0",
    author="MihaiStreames",
    description="DAW Discord Rich Presence - Show what you're making on your DAW in Discord",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/MihaiStreames/DAWRPC",
    packages=find_packages(),
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Development Status :: 4 - Beta",
        "Environment :: Win32 (MS Windows)",
        "Intended Audience :: End Users/Desktop",
        "Topic :: Multimedia :: Sound/Audio",
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
        '': ['*.json', '*.ico', '*.png'],
    },
)
