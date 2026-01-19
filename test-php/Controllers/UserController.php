<?php

namespace App\Controllers;

use App\Models\User;
use App\Services\UserService;

class UserController
{
    private UserService $userService;

    public function __construct(UserService $service)
    {
        $this->userService = $service;
    }

    public function getUser(int $id): User
    {
        return $this->userService->findUser($id);
    }
}
