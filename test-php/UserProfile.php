<?php

namespace App\Models;

class UserProfile
{
    private User $user;
    private array $preferences;

    public function __construct(User $user)
    {
        $this->user = $user;
        $this->preferences = [];
    }

    public function addPreference(string $key, string $value): void
    {
        $this->preferences[$key] = $value;
    }
}
