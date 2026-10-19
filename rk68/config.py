"""Profile and configuration management for RK68 on Linux."""

import json
import os
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Dict, List, Optional
from .constants import LightingMode


@dataclass
class KeyboardProfile:
    """Represents a saved keyboard configuration profile."""
    name: str = "default"
    lighting_mode: str = "wave"
    speed: int = 3
    brightness: int = 4
    color: str = "#FF5500"
    direction: int = 0
    key_remaps: Dict[str, str] = field(default_factory=dict)
    custom_rgb: Dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> "KeyboardProfile":
        return cls(
            name=data.get("name", "default"),
            lighting_mode=data.get("lighting_mode", "wave"),
            speed=data.get("speed", 3),
            brightness=data.get("brightness", 4),
            color=data.get("color", "#FF5500"),
            direction=data.get("direction", 0),
            key_remaps=data.get("key_remaps", {}),
            custom_rgb=data.get("custom_rgb", {}),
        )

    def get_color_rgb(self) -> tuple:
        """Parse hex color (#RRGGBB) into an (R, G, B) integer tuple."""
        clean = self.color.lstrip("#")
        if len(clean) != 6:
            return (255, 255, 255)
        try:
            return (int(clean[0:2], 16), int(clean[2:4], 16), int(clean[4:6], 16))
        except ValueError:
            return (255, 255, 255)


class ConfigManager:
    """Manages profile serialization and storage in user directory."""

    def __init__(self, config_dir: Optional[Path] = None):
        if config_dir is None:
            self.config_dir = Path.home() / ".config" / "rk68"
        else:
            self.config_dir = Path(config_dir)

        self.profiles_dir = self.config_dir / "profiles"
        self.settings_file = self.config_dir / "settings.json"
        self._ensure_dirs()

    def _ensure_dirs(self):
        self.profiles_dir.mkdir(parents=True, exist_ok=True)
        if not self.settings_file.exists():
            self._save_settings({"active_profile": "default"})

    def _load_settings(self) -> dict:
        try:
            with open(self.settings_file, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            return {"active_profile": "default"}

    def _save_settings(self, data: dict):
        with open(self.settings_file, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)

    def list_profiles(self) -> List[str]:
        """List all available profile names."""
        profiles = []
        for file in self.profiles_dir.glob("*.json"):
            profiles.append(file.stem)
        return sorted(profiles) if profiles else ["default"]

    def load_profile(self, name: str) -> KeyboardProfile:
        """Load a profile by name. Creates a default profile if not found."""
        profile_path = self.profiles_dir / f"{name}.json"
        if not profile_path.exists():
            profile = KeyboardProfile(name=name)
            self.save_profile(profile)
            return profile

        with open(profile_path, "r", encoding="utf-8") as f:
            data = json.load(f)
            return KeyboardProfile.from_dict(data)

    def save_profile(self, profile: KeyboardProfile):
        """Save a profile to disk."""
        profile_path = self.profiles_dir / f"{profile.name}.json"
        with open(profile_path, "w", encoding="utf-8") as f:
            json.dump(profile.to_dict(), f, indent=2)

    def delete_profile(self, name: str) -> bool:
        """Delete a profile by name."""
        if name == "default":
            return False
        profile_path = self.profiles_dir / f"{name}.json"
        if profile_path.exists():
            profile_path.unlink()
            return True
        return False

    def get_active_profile_name(self) -> str:
        return self._load_settings().get("active_profile", "default")

    def set_active_profile_name(self, name: str):
        settings = self._load_settings()
        settings["active_profile"] = name
        self._save_settings(settings)
