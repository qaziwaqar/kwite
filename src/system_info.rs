//! # System Information Collection Module
//!
//! This module provides functionality to collect system information for logging
//! and analytics purposes. It gathers OS details, hardware specifications, and
//! network interface information while respecting user privacy and security.
//!
//! ## Privacy Considerations
//!
//! This module collects system information that could be used to identify or
//! fingerprint a user's system. It should only be used when explicitly enabled
//! by configuration and with appropriate user consent.
//!
//! ## Information Collected
//!
//! - Operating System name and version
//! - System architecture (x86, ARM, etc.)
//! - Memory information (total/available RAM)
//! - CPU information (model and core count)
//! - Network interface MAC addresses (first available)
//! - External WAN IP address (public IP for analytics)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::Command;

/// System information structure for logging and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system name (e.g., "Windows", "macOS", "Linux")
    pub os_name: String,
    /// Operating system version
    pub os_version: String,
    /// System architecture (e.g., "x86_64", "aarch64")
    pub architecture: String,
    /// Total system memory in MB
    pub total_memory_mb: u64,
    /// Available system memory in MB
    pub available_memory_mb: u64,
    /// CPU model name
    pub cpu_model: String,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// MAC address of primary network interface (hashed for privacy)
    pub mac_address_hash: String,
    /// External WAN IP address for performance analytics
    pub ip_address: String,
    /// Timestamp when information was collected
    pub collected_at: String,
}

impl SystemInfo {
    /// Collect comprehensive system information
    ///
    /// This method gathers system information from various sources including
    /// the OS, hardware detection, and network interfaces. It handles errors
    /// gracefully by providing fallback values.
    ///
    /// ## Privacy Note
    ///
    /// MAC addresses are hashed using SHA-256 to protect user privacy while
    /// still allowing for basic device identification in analytics.
    pub fn collect() -> Self {
        Self {
            os_name: Self::get_os_name(),
            os_version: Self::get_os_version(),
            architecture: Self::get_architecture(),
            total_memory_mb: Self::get_total_memory_mb(),
            available_memory_mb: Self::get_available_memory_mb(),
            cpu_model: Self::get_cpu_model(),
            cpu_cores: Self::get_cpu_cores(),
            mac_address_hash: Self::get_mac_address_hash(),
            ip_address: Self::get_ip_address(),
            collected_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Get operating system name
    fn get_os_name() -> String {
        if cfg!(target_os = "windows") {
            "Windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macOS".to_string()
        } else if cfg!(target_os = "linux") {
            "Linux".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Get operating system version
    fn get_os_version() -> String {
        #[cfg(target_os = "windows")]
        {
            // On Windows, try to get version from registry or command
            if let Ok(output) = Command::new("cmd")
                .args(&["/C", "ver"])
                .output()
            {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string()
            } else {
                "Unknown Windows Version".to_string()
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, use sw_vers command
            if let Ok(output) = Command::new("sw_vers")
                .arg("-productVersion")
                .output()
            {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string()
            } else {
                "Unknown macOS Version".to_string()
            }
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, try to read from /etc/os-release
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("VERSION=") {
                        return line.split('=').nth(1)
                            .unwrap_or("Unknown")
                            .trim_matches('"')
                            .to_string();
                    }
                }
            }
            "Unknown Linux Version".to_string()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "Unknown Version".to_string()
        }
    }

    /// Get system architecture
    fn get_architecture() -> String {
        std::env::consts::ARCH.to_string()
    }

    /// Get total system memory in MB
    fn get_total_memory_mb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb / 1024; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "hw.memsize"])
                .output()
            {
                if let Ok(bytes_str) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                        return bytes / (1024 * 1024); // Convert bytes to MB
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Try PowerShell first for more reliable parsing
            if let Ok(output) = Command::new("powershell")
                .args(&["-Command", "(Get-CimInstance -Class Win32_ComputerSystem).TotalPhysicalMemory"])
                .output()
            {
                if let Ok(bytes_str) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                        return bytes / (1024 * 1024); // Convert bytes to MB
                    }
                }
            }
            
            // Fallback to wmic command with different format
            if let Ok(output) = Command::new("wmic")
                .args(&["computersystem", "get", "TotalPhysicalMemory", "/format:value"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.starts_with("TotalPhysicalMemory=") {
                        if let Some(bytes_str) = line.split('=').nth(1) {
                            if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                                return bytes / (1024 * 1024); // Convert bytes to MB
                            }
                        }
                    }
                }
            }
            
            // Another fallback using wmic with /value format
            if let Ok(output) = Command::new("wmic")
                .args(&["computersystem", "get", "TotalPhysicalMemory", "/value"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.starts_with("TotalPhysicalMemory=") {
                        if let Some(bytes_str) = line.split('=').nth(1) {
                            if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                                return bytes / (1024 * 1024); // Convert bytes to MB
                            }
                        }
                    }
                }
            }
        }

        0 // Fallback if unable to determine
    }

    /// Get available system memory in MB
    fn get_available_memory_mb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb / 1024; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Try PowerShell first for available memory
            if let Ok(output) = Command::new("powershell")
                .args(&["-Command", "(Get-CimInstance -Class Win32_OperatingSystem).FreePhysicalMemory * 1024"])
                .output()
            {
                if let Ok(bytes_str) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                        return bytes / (1024 * 1024); // Convert bytes to MB
                    }
                }
            }
            
            // Fallback to wmic command
            if let Ok(output) = Command::new("wmic")
                .args(&["OS", "get", "FreePhysicalMemory", "/format:value"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.starts_with("FreePhysicalMemory=") {
                        if let Some(kb_str) = line.split('=').nth(1) {
                            if let Ok(kb) = kb_str.trim().parse::<u64>() {
                                return kb / 1024; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, use vm_stat command
            if let Ok(output) = Command::new("vm_stat")
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut free_pages = 0u64;
                let mut page_size = 4096u64; // Default page size
                
                for line in output_str.lines() {
                    if line.starts_with("Pages free:") {
                        if let Some(pages_str) = line.split_whitespace().nth(2) {
                            if let Ok(pages) = pages_str.trim_end_matches('.').parse::<u64>() {
                                free_pages = pages;
                            }
                        }
                    } else if line.starts_with("Mach Virtual Memory Statistics:") {
                        // Try to get page size from sysctl
                        if let Ok(output) = Command::new("sysctl")
                            .args(&["-n", "hw.pagesize"])
                            .output()
                        {
                            if let Ok(page_str) = String::from_utf8(output.stdout) {
                                if let Ok(page) = page_str.trim().parse::<u64>() {
                                    page_size = page;
                                }
                            }
                        }
                    }
                }
                
                if free_pages > 0 {
                    return (free_pages * page_size) / (1024 * 1024); // Convert to MB
                }
            }
        }

        // For other platforms or if detection fails, estimate as 50% of total (rough approximation)
        Self::get_total_memory_mb() / 2
    }

    /// Get CPU model name
    fn get_cpu_model() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("model name") {
                        if let Some(model) = line.split(':').nth(1) {
                            return model.trim().to_string();
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "machdep.cpu.brand_string"])
                .output()
            {
                return String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("wmic")
                .args(&["cpu", "get", "name", "/value"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.starts_with("Name=") {
                        if let Some(name) = line.split('=').nth(1) {
                            return name.trim().to_string();
                        }
                    }
                }
            }
        }

        "Unknown CPU".to_string()
    }

    /// Get number of CPU cores
    fn get_cpu_cores() -> u32 {
        num_cpus::get() as u32
    }

    /// Get hashed MAC address for privacy-preserving identification
    fn get_mac_address_hash() -> String {
        use sha2::{Sha256, Digest};

        // Try to get MAC address from network interfaces
        if let Some(mac) = Self::get_primary_mac_address() {
            let mut hasher = Sha256::new();
            hasher.update(mac.as_bytes());
            format!("{:x}", hasher.finalize())
        } else {
            "unknown".to_string()
        }
    }

    /// Get primary network interface MAC address
    fn get_primary_mac_address() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            // Try to read from /sys/class/net interfaces
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    let interface_name = entry.file_name();
                    let interface_str = interface_name.to_string_lossy();
                    
                    // Skip loopback and virtual interfaces
                    if interface_str.starts_with("lo") || 
                       interface_str.starts_with("vir") ||
                       interface_str.starts_with("docker") {
                        continue;
                    }

                    let address_path = format!("/sys/class/net/{}/address", interface_str);
                    if let Ok(mac) = std::fs::read_to_string(&address_path) {
                        let mac = mac.trim();
                        if mac != "00:00:00:00:00:00" && !mac.is_empty() {
                            return Some(mac.to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("ifconfig")
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("ether") {
                        if let Some(mac) = line.split_whitespace().nth(1) {
                            if mac != "00:00:00:00:00:00" {
                                return Some(mac.to_string());
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("getmac")
                .args(&["/fo", "csv"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines().skip(1) { // Skip header
                    if let Some(mac) = line.split(',').next() {
                        let mac = mac.trim_matches('"');
                        if mac != "00-00-00-00-00-00" && !mac.is_empty() {
                            return Some(mac.replace('-', ":"));
                        }
                    }
                }
            }
        }

        None
    }

    /// Get external WAN IP address for performance analytics
    fn get_ip_address() -> String {
        // Try to get external IP address using curl command with multiple fallback services
        let services = [
            "https://api.ipify.org",
            "https://ifconfig.me/ip", 
            "https://ipinfo.io/ip",
            "https://httpbin.org/ip",
        ];

        for service in &services {
            if let Ok(output) = std::process::Command::new("curl")
                .args(&["-s", "--max-time", "5", "--connect-timeout", "3", service])
                .output()
            {
                if output.status.success() {
                    if let Ok(response) = String::from_utf8(output.stdout) {
                        let response = response.trim();
                        
                        // For httpbin.org/ip, extract IP from JSON response
                        if service.contains("httpbin.org") {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                                if let Some(origin) = json.get("origin") {
                                    if let Some(ip_str) = origin.as_str() {
                                        let ip = ip_str.trim();
                                        if Self::is_valid_wan_ip(ip) {
                                            return ip.to_string();
                                        }
                                    }
                                }
                            }
                        } else {
                            // Direct IP response from other services
                            if Self::is_valid_wan_ip(response) {
                                return response.to_string();
                            }
                        }
                    }
                }
            }
        }

        // Final fallback: try to get local IP address if external detection fails
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    let ip = local_addr.ip().to_string();
                    if ip != "0.0.0.0" && ip != "127.0.0.1" {
                        return ip;
                    }
                }
            }
        }

        "unknown".to_string()
    }

    /// Validate if a string is a valid WAN IP address (excludes private ranges)
    fn is_valid_wan_ip(ip: &str) -> bool {
        if ip.is_empty() || ip == "unknown" {
            return false;
        }
        
        // Basic IPv4 validation
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return false;
        }
        
        for part in parts {
            if let Ok(_num) = part.parse::<u8>() {
                // Valid IPv4 octet
                continue;
            } else {
                return false;
            }
        }
        
        // Exclude private/local ranges for WAN IP detection
        if ip.starts_with("10.") || 
           ip.starts_with("172.") || 
           ip.starts_with("192.168.") ||
           ip.starts_with("127.") ||
           ip == "0.0.0.0" {
            return false;
        }
        
        true
    }

    /// Convert to a formatted string for logging
    pub fn to_log_string(&self) -> String {
        format!(
            "OS: {} {}, Arch: {}, RAM: {}MB/{}MB, CPU: {} ({} cores), MAC Hash: {}, IP: {}, Collected: {}",
            self.os_name,
            self.os_version,
            self.architecture,
            self.available_memory_mb,
            self.total_memory_mb,
            self.cpu_model,
            self.cpu_cores,
            self.mac_address_hash,
            self.ip_address,
            self.collected_at
        )
    }

    /// Convert to HashMap for flexible logging backends
    pub fn to_fields(&self) -> HashMap<String, String> {
        let mut fields = HashMap::new();
        fields.insert("os_name".to_string(), self.os_name.clone());
        fields.insert("os_version".to_string(), self.os_version.clone());
        fields.insert("architecture".to_string(), self.architecture.clone());
        fields.insert("total_memory_mb".to_string(), self.total_memory_mb.to_string());
        fields.insert("available_memory_mb".to_string(), self.available_memory_mb.to_string());
        fields.insert("cpu_model".to_string(), self.cpu_model.clone());
        fields.insert("cpu_cores".to_string(), self.cpu_cores.to_string());
        fields.insert("mac_address_hash".to_string(), self.mac_address_hash.clone());
        fields.insert("ip_address".to_string(), self.ip_address.clone());
        fields.insert("collected_at".to_string(), self.collected_at.clone());
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_collection() {
        let info = SystemInfo::collect();
        
        // Verify basic fields are populated
        assert!(!info.os_name.is_empty());
        assert!(!info.architecture.is_empty());
        assert!(!info.mac_address_hash.is_empty());
        assert!(info.cpu_cores > 0);
        
        // Verify log string format
        let log_str = info.to_log_string();
        assert!(log_str.contains(&info.os_name));
        assert!(log_str.contains(&info.cpu_cores.to_string()));
    }

    #[test]
    fn test_system_info_fields() {
        let info = SystemInfo::collect();
        let fields = info.to_fields();
        
        assert!(fields.contains_key("os_name"));
        assert!(fields.contains_key("cpu_cores"));
        assert!(fields.contains_key("mac_address_hash"));
        assert!(fields.contains_key("ip_address"));
        assert_eq!(fields.len(), 10);
    }

    #[test]
    fn test_os_detection() {
        let os_name = SystemInfo::get_os_name();
        
        // Should detect one of the known operating systems
        assert!(
            os_name == "Windows" || 
            os_name == "macOS" || 
            os_name == "Linux" || 
            os_name == "Unknown"
        );
    }
}