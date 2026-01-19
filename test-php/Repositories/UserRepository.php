<?php

namespace App\Repositories;

use App\Models\Database;
use App\Models\User;

class UserRepository
{
    private Database $db;

    public function __construct(Database $db)
    {
        $this->db = $db;
    }

    public function find(int $id): User
    {
        // Implementation here
        return new User("name", "email");
    }

    public function save(User $user): bool
    {
        return $this->db->save($user);
    }
}
