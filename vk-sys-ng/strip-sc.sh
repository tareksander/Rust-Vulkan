#!/bin/bash

# Use the Vulkan Header scripts to generate a spec without VulkanSC

python3 scripts/stripAPI.py -input vk.xml -output vk-only.xml -keepAPI vulkan

