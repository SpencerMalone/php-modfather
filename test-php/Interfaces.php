<?php

namespace App\Models;

interface Authenticatable
{
    public function authenticate(): bool;
}

trait HasPermissions
{
    public function hasPermission(string $permission): bool
    {
        return true;
    }
}

class Database
{
    public function save(Model $model): bool
    {
        return true;
    }
}

class Country
{
    private string $name;
    private string $code;

    public function __construct(string $name, string $code)
    {
        $this->name = $name;
        $this->code = $code;
    }
}
