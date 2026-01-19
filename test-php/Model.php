<?php

namespace App\Models;

abstract class Model
{
    protected Database $db;

    public function save(): bool
    {
        return $this->db->save($this);
    }
}
