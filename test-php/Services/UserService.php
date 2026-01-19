<?php

namespace App\Services;

use App\Models\User;
use App\Models\UserProfile;
use App\Repositories\UserRepository;

class UserService
{
    private UserRepository $repository;

    public function __construct(UserRepository $repository)
    {
        $this->repository = $repository;
    }

    public function findUser(int $id): User
    {
        return $this->repository->find($id);
    }

    public function createUser(string $name): User
    {
        $user = new User($name, "email@example.com");
        return $user;
    }

    public function getUserProfile(User $user): UserProfile
    {
        return $user->getProfile();
    }
}
