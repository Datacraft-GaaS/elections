package com.zaz.fingers;

import android.content.Context;
import android.media.AudioManager;
import android.media.SoundPool;

import java.util.HashMap;
import java.util.Locale;

public class SoundPoolPlayer {
    private SoundPool mShortPlayer = null;

    private HashMap mSounds = new HashMap();

    String locale = Locale.getDefault().getLanguage();

    public SoundPoolPlayer(Context pContext) {
        // setup Soundpool
        if(locale.equals("zh")) {
            this.mShortPlayer = new SoundPool(1, AudioManager.STREAM_MUSIC, 0);

            mSounds.put(1, this.mShortPlayer.load(pContext, R.raw.two_thumb, 1));
            mSounds.put(2, this.mShortPlayer.load(pContext, R.raw.left_four, 1));
            mSounds.put(3, this.mShortPlayer.load(pContext, R.raw.right_four, 1));

            mSounds.put(5, this.mShortPlayer.load(pContext, R.raw.left_thumb, 1));
            mSounds.put(6, this.mShortPlayer.load(pContext, R.raw.left_index, 1));
            mSounds.put(7, this.mShortPlayer.load(pContext, R.raw.left_middle, 1));
            mSounds.put(8, this.mShortPlayer.load(pContext, R.raw.left_ring, 1));
            mSounds.put(9, this.mShortPlayer.load(pContext, R.raw.left_little, 1));


            mSounds.put(10, this.mShortPlayer.load(pContext, R.raw.right_thumb, 1));
            mSounds.put(11, this.mShortPlayer.load(pContext, R.raw.right_index, 1));
            mSounds.put(12, this.mShortPlayer.load(pContext, R.raw.right_middle, 1));
            mSounds.put(13, this.mShortPlayer.load(pContext, R.raw.right_ring, 1));
            mSounds.put(14, this.mShortPlayer.load(pContext, R.raw.right_little, 1));

            mSounds.put(15, this.mShortPlayer.load(pContext, R.raw.success, 1));
            mSounds.put(16, this.mShortPlayer.load(pContext, R.raw.timeout, 1));
            mSounds.put(17, this.mShortPlayer.load(pContext, R.raw.badquality, 1));

            mSounds.put(20, this.mShortPlayer.load(pContext, R.raw.press, 1));
        }else {
            this.mShortPlayer = new SoundPool(1, AudioManager.STREAM_MUSIC, 0);

            mSounds.put(1, this.mShortPlayer.load(pContext, R.raw.two_thumb_en, 1));
            mSounds.put(2, this.mShortPlayer.load(pContext, R.raw.left_four_en, 1));
            mSounds.put(3, this.mShortPlayer.load(pContext, R.raw.right_four_en, 1));

            mSounds.put(5, this.mShortPlayer.load(pContext, R.raw.left_thumb_en, 1));
            mSounds.put(6, this.mShortPlayer.load(pContext, R.raw.left_index_en, 1));
            mSounds.put(7, this.mShortPlayer.load(pContext, R.raw.left_middle_en, 1));
            mSounds.put(8, this.mShortPlayer.load(pContext, R.raw.left_ring_en, 1));
            mSounds.put(9, this.mShortPlayer.load(pContext, R.raw.left_little_en, 1));


            mSounds.put(10, this.mShortPlayer.load(pContext, R.raw.right_thumb_en, 1));
            mSounds.put(11, this.mShortPlayer.load(pContext, R.raw.right_index_en, 1));
            mSounds.put(12, this.mShortPlayer.load(pContext, R.raw.right_middle_en, 1));
            mSounds.put(13, this.mShortPlayer.load(pContext, R.raw.right_ring_en, 1));
            mSounds.put(14, this.mShortPlayer.load(pContext, R.raw.right_little_en, 1));

            mSounds.put(15, this.mShortPlayer.load(pContext, R.raw.success_en, 1));
            mSounds.put(16, this.mShortPlayer.load(pContext, R.raw.timeout_en, 1));
            mSounds.put(17, this.mShortPlayer.load(pContext, R.raw.badquality_en, 1));

            mSounds.put(20, this.mShortPlayer.load(pContext, R.raw.press_en, 1));
        }
    }

    public void playShortResource(int piResource) {
        int iSoundId = (Integer) mSounds.get(piResource);

        this.mShortPlayer.play(iSoundId, 0.99f, 0.99f, 0, 0, 1);
    }

    // Cleanup
    public void release() {
        this.mShortPlayer.release();

        this.mShortPlayer = null;
    }
}
