<?php

namespace App\Http\Controllers;

use Illuminate\View\View;

class WebviewController extends Controller
{
    public function privacyPolicy(): View
    {
        return view('privacy-policy');
    }
}
