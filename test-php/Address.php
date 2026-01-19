<?php

namespace App\Models;

class Address
{
    private string $street;
    private string $city;
    private Country $country;

    public function __construct(string $street, string $city, Country $country)
    {
        $this->street = $street;
        $this->city = $city;
        $this->country = $country;
    }
}
