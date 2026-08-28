package com.antiris;

import android.app.Application;
import android.content.Context;

import com.antiris.AntUSBCamera.utils.LanguageUtil;

public class MyApp extends Application {

    public static MyApp instance;

    @Override
    public void onCreate() {
        super.onCreate();
        instance = this;
        LanguageUtil.getInstance().applyLanguage(this);
    }

    @Override
    protected void attachBaseContext(Context base) {
        super.attachBaseContext(LanguageUtil.getInstance().applyLanguage(base));
    }
}