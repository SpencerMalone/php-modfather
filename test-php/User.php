<?php

namespace App\Models;

class User extends Model implements Authenticatable
{
    use HasPermissions;

    private string $name;
    private string $email;
    private ?Address $address;

    public function __construct(string $name, string $email)
    {
        $this->name = $name;
        $this->email = $email;
    }

    public function getProfile(): UserProfile
    {
        return new UserProfile($this);
    }
}
